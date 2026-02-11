use paste::paste;
use crate::cpu::disasm::Instruction;
use log::{debug, info, warn, error};

#[derive(Default)]
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
    pc: u16

}

macro_rules! get_set {
    ($reg:ident) => {
        paste! {
            pub fn [<get_ $reg>](&self) -> u8 {
                self.$reg
            }

            pub fn [<set_ $reg>](&mut self, val: u8) {
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


impl Registers {
    get_set!(a);
    get_set!(b);
    get_set!(c);
    get_set!(d);
    get_set!(e);
    get_set!(h);
    get_set!(l);
    get_set_dual!(b, c);
    get_set_dual!(d, e);
    get_set_dual!(h, l);
}

#[derive(Default)]
pub struct CPU {
    regs: Registers
}

impl CPU {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn step(&mut self, buf: &[u8]) {
        if let Some(instr) = Instruction::decode(buf, self.regs.pc as usize) {
            debug!("{:04X}: {:?}", self.regs.pc, instr);
            match instr {
                Instruction::NOP => println!("NOPped"),
                _ => todo!("{:?} not implemented", instr)
            }
            self.regs.pc += instr.get_size() as u16;
        } else {
            error!(
                "Invalid opcode {:02X} at {:04X}",
                buf[self.regs.pc as usize],
                self.regs.pc
            );
        }
    }
}
