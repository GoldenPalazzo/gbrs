mod squarechannel;
mod wavechannel;
mod noisechannel;
use noisechannel::NoiseChannel;
use squarechannel::SquareChannel;
use wavechannel::WaveChannel;
use alloc::vec::Vec;

pub struct Apu {
    ch1: SquareChannel,
    ch2: SquareChannel,
    ch3: WaveChannel,
    ch4: NoiseChannel,
    nr50: u8,
    nr51: u8,
    nr52: u8,
    enabled: bool,

    pub sample_rate_mcycles: u8,
    cur_cycles: u8,
    frame_sequencer: u8,
    samples: Vec<f32>,

    pub debug_disable: bool
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
            ch3: WaveChannel::default(),
            ch4: NoiseChannel::default(),
            nr50: 0,
            nr51: 0,
            nr52: 0,
            enabled: true,
            sample_rate_mcycles,
            cur_cycles: 0,
            frame_sequencer: 0,
            samples: Vec::new(),
            debug_disable: false
        }
    }

    pub fn step(&mut self, mcycles: u8) {
        if self.debug_disable {
            return;
        }
        self.ch1.step(mcycles);
        self.ch2.step(mcycles);
        self.ch3.step(mcycles);
        self.ch4.step(mcycles);
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
                    val |= MASTER_ONOFF_FLAG;
                }
                if self.ch1.enabled {
                    val |= MASTER_CH1_FLAG;
                }
                if self.ch2.enabled {
                    val |= MASTER_CH2_FLAG;
                }
                if self.ch3.enabled {
                    val |= MASTER_CH3_FLAG;
                }
                if self.ch4.enabled {
                    val |= MASTER_CH4_FLAG;
                }
                val
            }
            0xff25 => self.nr51,
            0xff24 => self.nr50,

            0xff10..=0xff14 => self.ch1.read(addr - 0xff10),
            0xff16..=0xff19 => self.ch2.read(addr - 0xff15),
            0xff1a..=0xff1e => self.ch3.read(addr),
            0xff20..=0xff23 => self.ch4.read(addr),

            0xff30..=0xff3f => self.ch3.read(addr),
            _ => 0xff,
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
                    self.ch3.enabled = false;
                    self.ch4.enabled = false;
                }
            }

            0xff25 => self.nr51 = data,
            0xff24 => self.nr50 = data,

            0xff10..=0xff14 => self.ch1.write(addr - 0xff10, data),
            0xff16..=0xff19 => self.ch2.write(addr - 0xff15, data),
            0xff1a..=0xff1e => self.ch3.write(addr, data),
            0xff20..=0xff23 => self.ch4.write(addr, data),

            0xff30..=0xff3f => self.ch3.write(addr, data),
            _ => {}
        }
    }

    pub fn drain_samples(&mut self) -> Vec<f32> {
        core::mem::take(&mut self.samples)
    }

    pub fn set_sample_rate(&mut self, sample_rate_khz: f32) {
        self.sample_rate_mcycles = (1048576.0 / sample_rate_khz) as u8;
    }

    pub fn divapu_tick(&mut self) {
        if self.debug_disable {
            return;
        }
        match self.frame_sequencer % 8 {
            0 | 2 | 4 | 6 => {
                self.ch1.clock_length();
                self.ch2.clock_length();
                self.ch3.clock_length();
                self.ch4.clock_length();
            }
            7 => {
                self.ch1.clock_envelope();
                self.ch2.clock_envelope();
                self.ch4.clock_envelope();
            }
            1 | 5 => self.ch1.clock_sweep(),
            _ => {}
        }
        self.frame_sequencer = self.frame_sequencer.wrapping_add(1);
    }

    fn mix(&self) -> (f32, f32) {
        let channels = [
            self.ch1.output(),
            self.ch2.output(),
            self.ch3.output(),
            self.ch4.output(),
        ];
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
