use crate::cpu::disasm::{Reg8, Reg16};
use paste::paste;

#[derive(Default)]
#[allow(dead_code)]
pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

macro_rules! get_set {
    ($reg:ident, $size:ident) => {
        paste! {
            pub fn [<get_ $reg>](&self) -> $size {
                self.$reg
            }

            pub fn [<set_ $reg>](&mut self, val: $size) {
                self.$reg = val;
            }
        }
    };
}

macro_rules! get_set_dual {
    ($reg1:ident, $reg2:ident) => {
        paste! {
            pub fn [<get_ $reg1 $reg2>](&self) -> u16 {
                u16::from_be_bytes([self.$reg1, self.$reg2])
            }

            pub fn [<set_ $reg1 $reg2>](&mut self, val: u16) {
                self.$reg1 = (val >> 8) as u8;
                self.$reg2 = val as u8;
            }
        }
    };
}

pub const FLAG_Z: u8 = 0x80;
pub const FLAG_N: u8 = 0x40;
pub const FLAG_H: u8 = 0x20;
pub const FLAG_C: u8 = 0x10;

#[allow(dead_code)]
impl Registers {
    get_set!(a, u8);
    get_set!(b, u8);
    get_set!(c, u8);
    get_set!(d, u8);
    get_set!(e, u8);
    get_set!(h, u8);
    get_set!(l, u8);
    get_set_dual!(a, f);
    get_set_dual!(b, c);
    get_set_dual!(d, e);
    get_set_dual!(h, l);
    get_set!(pc, u16);
    get_set!(sp, u16);

    pub fn get_reg8(&self, reg: &Reg8) -> u8 {
        match reg {
            Reg8::A => self.get_a(),
            Reg8::B => self.get_b(),
            Reg8::C => self.get_c(),
            Reg8::D => self.get_d(),
            Reg8::E => self.get_e(),
            Reg8::H => self.get_h(),
            Reg8::L => self.get_l(),
            Reg8::HLderef => panic!("HLderef requires memory access, use CPU helper"),
        }
    }

    pub fn set_reg8(&mut self, reg: &Reg8, val: u8) {
        match reg {
            Reg8::A => self.set_a(val),
            Reg8::B => self.set_b(val),
            Reg8::C => self.set_c(val),
            Reg8::D => self.set_d(val),
            Reg8::E => self.set_e(val),
            Reg8::H => self.set_h(val),
            Reg8::L => self.set_l(val),
            Reg8::HLderef => panic!("HLderef requires memory access, use CPU helper"),
        }
    }

    pub fn get_reg16(&self, reg: &Reg16) -> u16 {
        match reg {
            Reg16::BC => self.get_bc(),
            Reg16::DE => self.get_de(),
            Reg16::HL | Reg16::HLplus | Reg16::HLminus => self.get_hl(),
            Reg16::SP => self.get_sp(),
            Reg16::AF => self.get_af(),
            // _ => panic!("Complex Reg16 ({:?}) cannot be read directly", reg),
        }
    }

    pub fn set_reg16(&mut self, reg: &Reg16, val: u16) {
        match reg {
            Reg16::BC => self.set_bc(val),
            Reg16::DE => self.set_de(val),
            Reg16::HL => self.set_hl(val),
            Reg16::SP => self.set_sp(val),
            Reg16::AF => self.set_af(val & 0xFFF0), // I 4 bit bassi di F sono sempre 0
            _ => panic!("Complex Reg16 ({:?}) cannot be set directly", reg),
        }
    }

    pub fn get_flag(&self, flag: u8) -> bool {
        (self.f & flag) != 0
    }

    pub fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.f |= flag;
        } else {
            self.f &= !flag;
        }
    }
}
