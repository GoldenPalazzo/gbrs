#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Interrupt {
    VBlank  = 0b00001,
    LcdStat = 0b00010,
    Timer   = 0b00100,
    Serial  = 0b01000,
    Joypad  = 0b10000,
}

const IE_ADDR: u16 = 0xffff;
const IF_ADDR: u16 = 0xff0f;

#[derive(Default)]
pub struct InterruptController {
    pub ie: u8,
    pub if_: u8,
}

impl InterruptController {
    pub fn request(&mut self, int: Interrupt) {
        self.if_ |= int as u8;
    }

    /// Restituisce i bit degli interrupt attivi e abilitati (priorità LSB)
    pub fn pending(&self) -> u8 {
        self.ie & self.if_ & 0x1F
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            IE_ADDR => self.ie,
            IF_ADDR => self.if_ | 0xe0,
            _ => unreachable!()
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            IE_ADDR => self.ie = data,
            IF_ADDR => self.if_ = data & 0x1f,
            _ => unreachable!()
        }
    }
}
