#[derive(Default)]
pub struct SquareChannel {
    pub nr0: u8,
    pub nr1: u8,
    pub nr2: u8,
    pub nr3: u8,
    pub nr4: u8,

    pub enabled: bool,

    volume: u8,

    // length
    length_timer: u8,

    // envelope
    raising_envelope: bool,
    envelope_pace: u8,
    envelope_timer: u8,

    // sweep
    pub sweep_pace: u8,
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

    pub fn output(&self) -> f32 {
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

    pub fn step(&mut self, mcycles: u8) {
        self.period_timer -= mcycles as i16;
        if self.period_timer <= 0 {
            self.period = self.nr3 as u16 | ((self.nr4 as u16 & 0x7) << 8);
            self.period_timer += 2048 - self.period as i16;
            self.duty_pos = (self.duty_pos + 1) % 8;
        }
    }

    pub fn trigger(&mut self) {
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

    pub fn clock_length(&mut self) {
        let length_enable = (self.nr4 & 0x40) != 0;

        if length_enable && self.length_timer > 0 {
            self.length_timer -= 1;
            if self.length_timer == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn clock_envelope(&mut self) {
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

    pub fn clock_sweep(&mut self) {
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

    pub fn read(&self, offset: u16) -> u8 {
        match offset {
            0 => self.nr0 | 0x80,
            1 => self.nr1 | 0x3f,
            2 => self.nr2,
            3 => 0xff,
            4 => self.nr4 | 0xbf,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, offset: u16, data: u8) {
        match offset {
            0 => {
                self.nr0 = data | 0x80;
                self.sweep_pace = (self.nr0 >> 4) & 0x7;
            }
            1 => self.nr1 = data,
            2 => self.nr2 = data,
            3 => self.nr3 = data,
            4 => {
                self.nr4 = data;
                if data & 0x80 != 0 {
                    self.trigger();
                }
            }
            _ => unreachable!(),
        }
    }
}
