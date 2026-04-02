#[derive(Default)]
struct SquareChannel {
    nr0: u8,
    nr1: u8,
    nr2: u8,
    nr3: u8,
    nr4: u8,

    enabled: bool,

    volume: u8,

    // length
    length_timer: u8,

    // envelope
    raising_envelope: bool,
    envelope_pace: u8,
    envelope_timer: u8,

    // sweep
    sweep_pace: u8,
    raising_sweep: bool,
    sweep_single_step: u8,
    sweep_timer: u8,

    // main frequency
    period: u16, // 11 bits, overflows at $0x7ff
    period_timer: i16,
    duty_pos: u8,
}

impl SquareChannel {
    const DUTY_TABLE: [[u8; 8]; 4] = [
        [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
        [1, 0, 0, 0, 0, 0, 0, 1], // 25%
        [1, 1, 0, 0, 0, 0, 0, 1], // 50%
        [0, 1, 1, 1, 1, 1, 1, 0], // 75%
    ];

    fn output(&self) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        let duty = (self.nr1 >> 6) & 0x3;
        let bit = Self::DUTY_TABLE[duty as usize][self.duty_pos as usize];
        if bit == 1 {
            self.volume as f32 / 15.0
        } else {
            0.0
        }
    }

    fn step(&mut self, mcycles: u8) {
        self.period_timer -= mcycles as i16;
        if self.period_timer <= 0 {
            self.period = self.nr3 as u16 | ((self.nr4 as u16 & 0x7) << 8);
            self.period_timer += 2048 - self.period as i16;
            self.duty_pos = (self.duty_pos + 1) % 8;
        }
    }

    fn trigger(&mut self) {
        self.enabled = true;
        self.length_timer = 64 - (self.nr1 & 0x3f);
        self.period = self.nr3 as u16 | ((self.nr4 as u16 & 0x7) << 8);
        self.period_timer = 2048 - self.period as i16;
        self.volume = (self.nr2 >> 4) & 0xf;
        self.raising_envelope = self.nr2 & 0x08 != 0;
        self.envelope_pace = self.nr2 & 0x07;
        self.envelope_timer = self.envelope_pace;
        self.sweep_pace = (self.nr0 >> 4) & 0x7;
        self.sweep_single_step = self.nr0 & 0x7;
        self.raising_sweep = (self.nr0 & 0x08) == 0;
    }

    fn clock_length(&mut self) {
        let length_enable = (self.nr4 & 0x40) != 0;

        if length_enable && self.length_timer > 0 {
            self.length_timer -= 1;
            if self.length_timer == 0 {
                self.enabled = false;
            }
        }
    }

    fn clock_envelope(&mut self) {
        if self.envelope_pace == 0 {
            return;
        }
        if self.envelope_timer > 0 {
            self.envelope_timer -= 1;
        }
        if self.envelope_timer == 0 {
            self.envelope_timer = self.envelope_pace;
            if self.raising_envelope && self.volume < 15 {
                self.volume += 1;
            } else if !self.raising_envelope && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }

    fn clock_sweep(&mut self) {
        if self.sweep_pace == 0 {
            return;
        }
        if self.sweep_timer > 0 {
            self.sweep_timer -= 1;
        }

        if self.sweep_timer == 0 {
            self.sweep_timer = self.sweep_pace;
            let new_period = if self.raising_sweep {
                self.period + self.period / 2u16.pow(self.sweep_single_step as u32)
            } else {
                self.period - self.period / 2u16.pow(self.sweep_single_step as u32)
            };
            if new_period > 0x7ff || (!self.raising_sweep && new_period == 0) {
                self.enabled = false;
            } else {
                self.nr3 = new_period as u8;
                self.nr4 = (self.nr4 & 0xF8) | ((new_period >> 8) as u8 & 0x7);
                self.period = new_period;
                self.period_timer = 2048 - self.period as i16;
            }
        }
    }
}

pub struct Apu {
    ch1: SquareChannel,
    ch2: SquareChannel,
    nr50: u8,
    nr51: u8,
    nr52: u8,
    enabled: bool,

    wave_pattern: [u8; 16],

    pub sample_rate_mcycles: u8,
    cur_cycles: u8,
    frame_sequencer: u8,
    samples: Vec<f32>,
}

const MASTER_ONOFF_FLAG: u8 = 0x80;
const MASTER_CH4_FLAG: u8 = 0x08;
const MASTER_CH3_FLAG: u8 = 0x04;
const MASTER_CH2_FLAG: u8 = 0x02;
const MASTER_CH1_FLAG: u8 = 0x01;

impl Apu {
    pub fn new(sample_rate_mcycles: u8) -> Self {
        Self {
            ch1: SquareChannel::default(),
            ch2: SquareChannel::default(),
            nr50: 0,
            nr51: 0,
            nr52: 0,
            enabled: true,
            wave_pattern: [0u8; 16],
            sample_rate_mcycles,
            cur_cycles: 0,
            frame_sequencer: 0,
            samples: Vec::new(),
        }
    }

    pub fn step(&mut self, mcycles: u8) {
        self.ch1.step(mcycles);
        self.ch2.step(mcycles);
        self.cur_cycles += mcycles;
        if self.cur_cycles >= self.sample_rate_mcycles {
            let mixed = self.mix();
            self.samples.push(mixed.0);
            self.samples.push(mixed.1);
            self.cur_cycles -= self.sample_rate_mcycles;
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff26 => {
                let mut val = 0x70; // I bit 4-6 sono solitamente 1
                if self.enabled {
                    val |= 0x80;
                }
                if self.ch1.enabled {
                    val |= 0x01;
                }
                if self.ch2.enabled {
                    val |= 0x02;
                }
                // if self.ch3.enabled {
                //     val |= 0x04;
                // }
                // if self.ch4.enabled {
                //     val |= 0x08;
                // }
                val
            }
            0xff25 => self.nr51,
            0xff24 => self.nr50,

            0xff10 => self.ch1.nr0 | 0x80,
            0xff11 => self.ch1.nr1 | 0x3f,
            0xff12 => self.ch1.nr2,
            0xff13 => 0xff,
            0xff14 => self.ch1.nr4 | 0xbf,

            0xff16 => self.ch2.nr1 | 0x3f,
            0xff17 => self.ch2.nr2,
            0xff18 => 0xff,
            0xff19 => self.ch2.nr4 | 0xbf,

            0xff30..=0xff3f => self.wave_pattern[addr as usize - 0xff30],
            _ => 0xff,
            // _ => todo!("Invalid read at 0x{:04X}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xff26 => {
                let old_enabled = self.enabled;
                self.enabled = (data & 0x80) != 0;

                // Se l'APU viene spenta (Power Off)
                if old_enabled && !self.enabled {
                    // self.reset_registers();
                    self.ch1.enabled = false;
                    self.ch2.enabled = false;
                    // self.ch3.enabled = false;
                    // self.ch4.enabled = false;
                }
            }

            0xff25 => self.nr51 = data,
            0xff24 => self.nr50 = data,

            0xff10 => {
                self.ch1.nr0 = data | 0x80;
                self.ch1.sweep_pace = (self.ch1.nr0 >> 4) & 0x7;
            }
            0xff11 => self.ch1.nr1 = data,
            0xff12 => self.ch1.nr2 = data,
            0xff13 => self.ch1.nr3 = data,
            0xff14 => {
                self.ch1.nr4 = data;
                if data & 0x80 != 0 {
                    self.ch1.trigger();
                }
            }

            0xff16 => self.ch2.nr1 = data,
            0xff17 => self.ch2.nr2 = data,
            0xff18 => self.ch2.nr3 = data,
            0xff19 => {
                self.ch2.nr4 = data;
                if data & 0x80 != 0 {
                    self.ch2.trigger();
                }
            }

            0xff30..=0xff3f => self.wave_pattern[addr as usize - 0xff30] = data,
            _ => {} // _ => todo!("Invalid read at 0x{:04X}", addr),
        }
    }

    pub fn drain_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.samples)
    }

    pub fn set_sample_rate(&mut self, sample_rate_khz: f32) {
        self.sample_rate_mcycles = (1048576.0 / sample_rate_khz) as u8;
    }

    pub fn divapu_tick(&mut self) {
        match self.frame_sequencer % 8 {
            0 | 2 | 4 | 6 => {
                self.ch1.clock_length();
                self.ch2.clock_length();
            }
            7 => {
                self.ch1.clock_envelope();
                self.ch2.clock_envelope();
            }
            1 | 5 => self.ch1.clock_sweep(),
            _ => {}
        }
        self.frame_sequencer = self.frame_sequencer.wrapping_add(1);
    }

    fn mix(&self) -> (f32, f32) {
        let channels = [self.ch1.output(), self.ch2.output(), 0., 0.];
        let mut left = 0.;
        let mut right = 0.;

        for (i, &sample) in channels.iter().enumerate() {
            if self.nr51 & (0x10 << i) != 0 {
                left += sample;
            }
            if self.nr51 & (1 << i) != 0 {
                right += sample;
            }
        }

        left /= 4.;
        right /= 4.;

        left *= ((self.nr50 >> 4) & 0x7) as f32 / 7.;
        right *= (self.nr50 & 0x7) as f32 / 7.;

        (left, right)
    }
}
