#[derive(Default)]
pub struct WaveChannel {
    pub nr0: u8, // dac on/off
    pub nr1: u8, // initial length timer
    pub nr2: u8, // volume (bits 6-5 shift volume by their amount)
    pub nr3: u8,
    pub nr4: u8,

    pub enabled: bool,

    volume: u8,

    length_timer: u8,

    wave_pattern: [u8; 16],
    wave_cur_nibble: u8,

    period: u16, // 11 bits, overflows at $0x7ff
    period_timer: i16,
}

impl WaveChannel {
    const VOLUME_SHIFT: [u8; 4] = [4, 0, 1, 2];

    pub fn output(&self) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        let cur_byte = self.wave_pattern[self.wave_cur_nibble as usize / 2];
        let val = if self.wave_cur_nibble.is_multiple_of(2) {
            cur_byte >> 4 & 0xf
        } else {
            cur_byte & 0xf
        };
        let shifted = val >> Self::VOLUME_SHIFT[self.volume as usize];
        shifted as f32 / 15.
    }

    pub fn step(&mut self, mcycles: u8) {
        self.period_timer -= mcycles as i16 * 2;
        if self.period_timer <= 0 {
            self.period = self.nr3 as u16 | ((self.nr4 as u16 & 0x7) << 8);
            self.period_timer += 2048 - self.period as i16;
            self.wave_cur_nibble = (self.wave_cur_nibble + 1) % 32;
        }
    }

    pub fn trigger(&mut self) {
        self.enabled = true;
        if self.length_timer == 0 {
            self.length_timer = 255 - self.nr1;
        }
        self.period = self.nr3 as u16 | ((self.nr4 as u16 & 0x7) << 8);
        self.period_timer = 2048 - self.period as i16;
        self.volume = (self.nr2 >> 5) & 0x3;
        self.wave_cur_nibble = 0;
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

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff1a => self.nr0 | 0x80,
            0xff1b => self.nr1 | 0x3f,
            0xff1c => self.nr2,
            0xff1d => 0xff,
            0xff1e => self.nr4 | 0xbf,
            0xff30..=0xff3f => self.wave_pattern[addr as usize - 0xff30],
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xff1a => {
                self.nr0 = data | 0x7f;
                self.enabled = data & 0x80 != 0;
            }
            0xff1b => self.nr1 = data,
            0xff1c => self.nr2 = data,
            0xff1d => self.nr3 = data,
            0xff1e => {
                self.nr4 = data;
                if data & 0x80 != 0 {
                    self.trigger();
                }
            }
            0xff30..=0xff3f => self.wave_pattern[addr as usize - 0xff30] = data,
            _ => unreachable!(),
        }
    }
}
