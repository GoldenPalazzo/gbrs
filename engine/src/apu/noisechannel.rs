#[derive(Default)]
pub struct NoiseChannel {
    pub nr1: u8, // initial length timer
    pub nr2: u8, // volume (bits 6-5 shift volume by their amount)
    pub nr3: u8,
    pub nr4: u8,

    pub enabled: bool,

    volume: u8,

    length_timer: u8,

    raising_envelope: bool,
    envelope_pace: u8,
    envelope_timer: u8,

    period: i32, // 11 bits, overflows at $0x7ff

    lsfr: u16,
}

impl NoiseChannel {
    pub fn output(&self) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        if self.lsfr & 1 == 1 {
            self.volume as f32 / 15.0
        } else {
            0.0
        }
    }

    pub fn step(&mut self, mcycles: u8) {
        self.period = self.period.wrapping_sub(mcycles as i32);
        if self.period <= 0 {
            self.period += self.evaluate_period();
            let new_bit = (!(self.lsfr >> 1 ^ self.lsfr)) & 1;
            self.lsfr = self.lsfr >> 1 | new_bit << 14;
            if self.nr3 & 0x08 != 0 {
                self.lsfr = (self.lsfr & 0xff7f) | new_bit << 6;
            }
        }
    }

    pub fn trigger(&mut self) {
        self.enabled = true;
        if self.length_timer == 0 {
            self.length_timer = 64 - (self.nr1 & 0x3f);
        }

        self.period = self.evaluate_period();
        self.volume = (self.nr2 >> 4) & 0xf;
        self.raising_envelope = self.nr2 & 0x08 != 0;
        self.envelope_pace = self.nr2 & 0x07;
        self.envelope_timer = self.envelope_pace;
        self.lsfr = 0;
    }

    fn evaluate_period(&self) -> i32 {
        let divider = (self.nr3 & 7) as i32;
        let shift = ((self.nr3 >> 4) & 0xf) as u32;
        if divider == 0 {
            2i32.pow(shift + 1)
        } else {
            divider * 2i32.pow(shift + 2)
        }
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

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff20 => self.nr1 | 0x3f,
            0xff21 => self.nr2,
            0xff22 => 0xff,
            0xff23 => self.nr4 | 0xbf,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xff20 => self.nr1 = data,
            0xff21 => self.nr2 = data,
            0xff22 => self.nr3 = data,
            0xff23 => {
                self.nr4 = data;
                if data & 0x80 != 0 {
                    self.trigger();
                }
            }
            _ => unreachable!(),
        }
    }
}
