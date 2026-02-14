use paste::paste;
use crate::cpu::disasm::{
    Instruction,
    Operand,
    Reg8, Reg16,
    Condition
};
use crate::MemoryBus;
use log::{debug, info, warn, error};

#[derive(Default)]
struct Registers {
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
}

#[derive(Default)]
pub struct CPU {
    regs: Registers
}

impl CPU {
    pub fn new() -> Self {
        let mut new = Self::default();
        new.regs.set_pc(0x100);
        new
    }

    pub fn step(&mut self, mem_bus: &MemoryBus) {
        if let Some(instr) = Instruction::decode(mem_bus.read(self.regs.pc)) {
            debug!("{:04X}: {:?}", self.regs.pc, instr);
            self.execute_instruction(mem_bus, &instr);
        } else {
            error!(
                "Invalid opcode {:02X} at {:04X}",
                mem_bus.read(self.regs.pc),
                self.regs.pc
            );
        }
    }

    fn execute_instruction(&mut self, mem_bus: &MemoryBus, instr: &Instruction) {
        let mut new_pc = self.regs.get_pc() + instr.get_size() as u16;
        match instr {
            Instruction::NOP => (),
            Instruction::JP(cond, op) => 'jp: {
                if !self.check_cond(cond) {break 'jp};
                match op {
                    Operand::Imm16 => {
                        new_pc = u16::from_le_bytes([
                            mem_bus.read(self.regs.pc+1),
                            mem_bus.read(self.regs.pc+2)
                        ]);
                    },
                    Operand::Reg16(reg) => {
                        match reg {
                            Reg16::HL => new_pc = self.regs.get_hl(),
                            _ => unreachable!("Non-existing instruction {:?}", instr)
                        }
                    }
                    _ => unreachable!("Non-existing instruction {:?}", instr)
                }
            }
            _ => todo!("{:?} not implemented", instr)
        }
        self.regs.set_pc(new_pc);
        debug!("New PC: {:04X}", self.regs.get_pc());
    }

    fn check_cond(&self, cond: &Option<Condition>) -> bool {
        match cond {
            None => true,
            Some(Condition::NonZero) => {
                ((self.regs.get_af() & 0x80) >> 7) == 0
            },
            Some(Condition::Zero) => {
                ((self.regs.get_af() & 0x80) >> 7) == 1
            },
            Some(Condition::NonCarry) => {
                ((self.regs.get_af() & 0x10) >> 4) == 0
            },
            Some(Condition::Carry) => {
                ((self.regs.get_af() & 0x10) >> 4) == 1
            },
            Some(_) => todo!("{:?} not implemented", cond)
        }
    }
}
