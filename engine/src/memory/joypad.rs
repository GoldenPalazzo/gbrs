use crate::memory::interrupts::Interrupt;

const BTNS_FLAG: u8 = 0x20;
const DPAD_FLAG: u8 = 0x10;

pub struct Joypad {
    buttons: u8,
    dpad: u8,
    select: u8,

    last_state: u8,
}

impl Default for Joypad {
    fn default() -> Self {
        Self {
            buttons: 0xFF,
            dpad: 0xFF,
            select: 0,
            last_state: 0,
        }
    }
}

impl Joypad {
    pub fn read(&self, addr: u16) -> u8 {
        if addr != 0xff00 {
            unreachable!();
        }
        (self.select & 0xf0) | self.get_lower()
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        if addr != 0xff00 {
            unreachable!();
        }
        self.select = data & 0xf0;
    }

    pub fn set_buttons(
        &mut self,
        a: bool,
        b: bool,
        select: bool,
        start: bool,
    ) -> Option<Interrupt> {
        let old_btns = self.buttons;
        self.buttons =
            !(u8::from(start) << 3 | u8::from(select) << 2 | u8::from(b) << 1 | u8::from(a)) & 0xf;
        match old_btns & !self.buttons != 0 && self.select & BTNS_FLAG != 0 {
            true => Some(Interrupt::Joypad),
            false => None,
        }
    }

    pub fn set_dpad(&mut self, right: bool, left: bool, up: bool, down: bool) -> Option<Interrupt> {
        let old_dpad = self.dpad;
        self.dpad =
            !(u8::from(down) << 3 | u8::from(up) << 2 | u8::from(left) << 1 | u8::from(right))
                & 0xf;
        match old_dpad & !self.dpad != 0 && self.select & DPAD_FLAG != 0 {
            true => Some(Interrupt::Joypad),
            false => None,
        }
    }

    fn get_lower(&self) -> u8 {
        0xf & match (self.select & BTNS_FLAG == 0, self.select & DPAD_FLAG == 0) {
            (false, false) => 0xf,
            (true, false) => self.buttons,
            (false, true) => self.dpad,
            (true, true) => self.buttons & self.dpad,
        }
    }
}
