use paste::paste;
#[allow(unused_imports)]
use crate::cpu::disasm::{
    Instruction,
    Operand,
    Reg8, Reg16,
    Condition
};
use crate::MemoryBus;
#[allow(unused_imports)]
use log::{debug, info, warn, error};

#[derive(Default)]
#[allow(dead_code)]
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

    pub fn print_state(&self) {
        let r = &self.regs;
        // Recuperiamo i flag singolarmente per una lettura rapida
        let f = r.get_af() as u8;
        let z = if f & 0x80 != 0 { 'Z' } else { '-' };
        let n = if f & 0x40 != 0 { 'N' } else { '-' };
        let h = if f & 0x20 != 0 { 'H' } else { '-' };
        let c = if f & 0x10 != 0 { 'C' } else { '-' };

        debug!("--- CPU State ---");
        debug!(
            "PC: {:04X}  SP: {:04X}  AF: {:04X} [{}{}{}{}]",
            r.get_pc(), r.get_sp(), r.get_af(), z, n, h, c
        );
        debug!(
            "BC: {:04X}  DE: {:04X}  HL: {:04X}",
            r.get_bc(), r.get_de(), r.get_hl()
        );
        debug!("-----------------");
    }

    pub fn step(&mut self, mem_bus: &mut MemoryBus) {
        let opcode = mem_bus.read(self.regs.pc);
        if let Some(instr) = Instruction::decode(opcode) {
            debug!("{:04X}: {:?}", self.regs.pc, instr);
            self.execute_instruction(mem_bus, &instr);
        } else {
            error!(
                "Invalid opcode 0x{:02X} at 0x{:04X}",
                opcode,
                self.regs.pc
            );
        }
    }

    fn execute_instruction(&mut self, mem_bus: &mut MemoryBus, instr: &Instruction) {
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
            Instruction::LD(op1, op2) => {
                match (op1, op2) {
                    // Block 0
                    (Operand::Reg16(reg), Operand::Imm16) => {
                        let val = u16::from_le_bytes([
                            mem_bus.read(self.regs.pc+1),
                            mem_bus.read(self.regs.pc+2),
                        ]);
                        self.regs.set_reg16(reg, val);
                    }
                    (Operand::AddrIndirect(dst), Operand::Reg8(Reg8::A)) => {
                        mem_bus.write8(
                            self.regs.get_reg16(dst),
                            self.regs.get_a()
                        );
                        match dst {
                            Reg16::HLplus => self.regs.set_hl(self.regs.get_hl()+1),
                            Reg16::HLminus => self.regs.set_hl(self.regs.get_hl()-1),
                            _ => ()
                        }
                    }
                    (Operand::Reg8(Reg8::A), Operand::AddrIndirect(src)) => {
                        self.regs.set_reg8(
                            &Reg8::A,
                            mem_bus.read(self.regs.get_reg16(src))
                        );
                        match src {
                            Reg16::HLplus => self.regs.set_hl(self.regs.get_hl()+1),
                            Reg16::HLminus => self.regs.set_hl(self.regs.get_hl()-1),
                            _ => ()
                        }
                    }
                    (Operand::AddrDirect16, Operand::Reg16(Reg16::SP)) => {
                        let addr = u16::from_le_bytes([
                            mem_bus.read(self.regs.pc+1),
                            mem_bus.read(self.regs.pc+2),
                        ]);
                        mem_bus.write16(
                            addr,
                            self.regs.get_reg16(&Reg16::SP),
                        );
                    }

                    // ----
                    (Operand::Reg8(dst), Operand::Reg8(src)) => {
                        self.regs.set_reg8(dst, self.regs.get_reg8(src));
                    }
                    (Operand::Reg8(dst), Operand::Imm8) => {
                        self.regs.set_reg8(dst, mem_bus.read(self.regs.pc+1));
                    }
                    _ => todo!("need to implement {:?} variant", instr)
                }
            }
            _ => todo!("{:?} (0x{:02X}) not implemented",
                instr, mem_bus.read(self.regs.pc))
        }
        self.regs.set_pc(new_pc);
        self.print_state();
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
            // Some(_) => todo!("{:?} not implemented", cond)
        }
    }
}
