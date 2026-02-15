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

const FLAG_Z: u8 = 0x80;
const FLAG_N: u8 = 0x40;
const FLAG_H: u8 = 0x20;
const FLAG_C: u8 = 0x10;

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
        if value { self.f |= flag; }
        else { self.f &= !flag; }
    }
}

#[derive(Default)]
pub struct CPU {
    pub regs: Registers,
    ime: bool,
    ie: u8,
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
        let opcode = self.read_byte(mem_bus);
        if let Some(instr) = Instruction::decode(opcode) {
            debug!("{:04X}: {:?}", self.regs.pc.wrapping_sub(1), instr);
            self.execute_instruction(mem_bus, &instr);
        } else {
            error!(
                "Invalid opcode 0x{:02X} at 0x{:04X}",
                opcode,
                self.regs.pc.wrapping_sub(1)
            );
        }
    }

    fn jump(&mut self, bus: &MemoryBus, cond: &Option<Condition>, op: &Operand, rel: bool) {
        let addr = self.get_operand_value(bus, op);
        if !self.check_cond(cond) {return;}
        if rel {
            let offset = addr as u8 as i8;
            let new_pc = (self.regs.pc as i32).wrapping_add(offset as i32) as u16;
            self.regs.set_pc(new_pc);
        } else {
            self.regs.set_pc(addr);
        }
    }

    fn load(&mut self, bus: &mut MemoryBus, dst: &Operand, src: &Operand) {
        let val = self.get_operand_value(bus, src);
        self.set_operand_value(bus, dst, val);
    }

    fn add(&mut self, bus: &mut MemoryBus, dst: &Operand, src: &Operand, carry: bool) {
        let src_val = self.get_operand_value(bus, src);
        let dst_val = self.get_operand_value(bus, dst);
        let carry = if carry { 1u8 } else { 0u8 };
        let res = dst_val.wrapping_add(src_val).wrapping_add(carry as u16);
        self.set_operand_value(bus, dst, res);
        self.regs.set_flag(FLAG_N, false);
        match (dst, src) {
            (Operand::Reg16(Reg16::SP), Operand::Imm8)
            | (Operand::Reg16(Reg16::HL), Operand::Reg16(_)) => {
                self.regs.set_flag(FLAG_Z, res == 0);
                self.regs.set_flag(FLAG_C, res < dst_val);
                self.regs.set_flag(
                    FLAG_H,
                    (src_val & 0xfff) + (dst_val & 0xfff) + carry as u16 > 0xfff
                )
            }
            (Operand::Reg8(Reg8::A), _) => {
                self.regs.set_flag(FLAG_Z, (res as u8) == 0);
                self.regs.set_flag(FLAG_C, (res as u8) < (dst_val as u8));
                self.regs.set_flag(
                    FLAG_H,
                    (src_val as u8 & 0xf) + (dst_val as u8 & 0xf) + carry > 0xf
                )
            }
            _ => unreachable!()
        }
    }

    fn inc(&mut self, bus: &mut MemoryBus, dst: &Operand) {
        let dst_val = self.get_operand_value(bus, dst);
        let res = dst_val.wrapping_add(1);
        self.set_operand_value(bus, dst, res);
        self.regs.set_flag(FLAG_N, false);
        match dst {
            Operand::Reg16(_) => {
            }
            Operand::Reg8(_) => {
                self.regs.set_flag(FLAG_Z, (res as u8) == 0);
                self.regs.set_flag(
                    FLAG_H,
                    (dst_val as u8 & 0xf) + 1 > 0xf
                );
            }
            _ => unreachable!()
        }
    }

    fn dec(&mut self, bus: &mut MemoryBus, dst: &Operand) {
        let dst_val = self.get_operand_value(bus, dst);
        let res = dst_val.wrapping_sub(1);
        self.set_operand_value(bus, dst, res);
        self.regs.set_flag(FLAG_N, true);
        match dst {
            Operand::Reg16(_) => {
            }
            Operand::Reg8(_) => {
                self.regs.set_flag(FLAG_Z, (res as u8) == 0);
                self.regs.set_flag(
                    FLAG_H,
                    (dst_val as u8 & 0xf) == 0
                );
            }
            _ => unreachable!()
        }
    }

    fn rotate_acc(&mut self, carry: bool, right: bool) {
        assert_eq!(right, true);
        let mut val = self.regs.get_a();
        let old_carry = if self.regs.get_flag(FLAG_C) {0xff} else {0xfe};
        self.regs.set_flag(FLAG_C, (val & 0x80) > 0);
        val = val.rotate_left(1);
        if !carry { val &= old_carry; }
        self.regs.set_a(val);
        self.regs.set_flag(FLAG_Z, false);
        self.regs.set_flag(FLAG_H, false);
        self.regs.set_flag(FLAG_N, false);
    }

    fn complement_acc(&mut self) { 
        let val = self.regs.get_a();
        self.regs.set_a(!val);
        self.regs.set_flag(FLAG_H, true);
        self.regs.set_flag(FLAG_N, true);
    }

    fn mod_carry(&mut self, compl: bool) {
        let f = !compl || !self.regs.get_flag(FLAG_C);
        self.regs.set_flag(FLAG_H, false);
        self.regs.set_flag(FLAG_N, false);
        self.regs.set_flag(FLAG_C, f);
    }

    fn execute_instruction(&mut self, bus: &mut MemoryBus, instr: &Instruction) {
        match instr {
            Instruction::NOP => (),
            Instruction::JP(cond, op) => self.jump(bus, cond, op, false),
            Instruction::LD(dst, src) => self.load(bus, dst, src),
            Instruction::INC(reg) => self.inc(bus, reg),
            Instruction::DEC(reg) => self.dec(bus, reg),
            Instruction::ADD(dst, src) => self.add(bus, dst, src, false),
            Instruction::ADC(dst, src) => self.add(bus, dst, src, true),
            // Instruction::SUB(dst, src) => self.add(bus, dst, src, true),
            Instruction::JR(cond, op) => self.jump(bus, cond, op, true),


            Instruction::RLCA => self.rotate_acc(true, false),
            Instruction::RRCA => self.rotate_acc(true, true),
            Instruction::RLA => self.rotate_acc(false, false),
            Instruction::RRA => self.rotate_acc(false, true),
            Instruction::DAA => todo!("Implementing BCD instrs later"),
            Instruction::CPL => self.complement_acc(),
            Instruction::SCF => self.mod_carry(false),
            Instruction::CCF => self.mod_carry(true),
            Instruction::STOP => todo!("Should implement low pow mode"),
            Instruction::HALT => todo!("Should implement low pow mode and interrupts"),


            Instruction::DI => self.ime = false,
            Instruction::EI => self.ime = true,
            Instruction::Hardlock => panic!("Hardlocked!"),
            _ => todo!("{:?} (0x{:02X}) not implemented",
                instr, bus.read(self.regs.pc))
        }
        self.print_state();
    }

    fn check_cond(&self, cond: &Option<Condition>) -> bool {
        match cond {
            None => true,
            Some(Condition::NonZero) => !self.regs.get_flag(FLAG_Z),
            Some(Condition::Zero) => self.regs.get_flag(FLAG_Z),
            Some(Condition::NonCarry) => !self.regs.get_flag(FLAG_C),
            Some(Condition::Carry) => self.regs.get_flag(FLAG_C),
            // Some(_) => todo!("{:?} not implemented", cond)
        }
    }

    fn get_operand_value(&mut self, bus: &MemoryBus, op: &Operand) -> u16 {
        match op {
            Operand::Reg8(r) => {
                if let Reg8::HLderef = r {
                    bus.read(self.regs.get_hl()) as u16
                } else {
                    self.regs.get_reg8(r) as u16
                }
            }
            Operand::Imm8 => self.read_byte(bus) as u16,
            Operand::Reg16(r) => {
                let val = self.regs.get_reg16(r);
                match r {
                    Reg16::HLplus => self.regs.set_hl(val.wrapping_add(1)),
                    Reg16::HLminus => self.regs.set_hl(val.wrapping_sub(1)),
                    _ => {}
                }
                val
            }
            Operand::Imm16 => self.read_word(bus),
            Operand::AddrIndirect(r) => {
                let addr = self.regs.get_reg16(r);
                match r {
                    Reg16::HLplus => self.regs.set_hl(addr.wrapping_add(1)),
                    Reg16::HLminus => self.regs.set_hl(addr.wrapping_sub(1)),
                    _ => {}
                }
                bus.read(addr) as u16
            }
            Operand::AddrDirect16 => bus.read(self.read_word(bus)) as u16
        }
    }

    fn set_operand_value(&mut self, bus: &mut MemoryBus, op: &Operand, value: u16) {
        match op {
            Operand::Imm8
                | Operand::Imm16
                | Operand::Reg16(Reg16::HLplus)
                | Operand::Reg16(Reg16::HLminus) => unreachable!(),
            Operand::Reg8(Reg8::HLderef) => bus.write8(self.regs.get_hl(), value as u8),
            Operand::Reg8(r) => self.regs.set_reg8(r, value as u8),
            Operand::Reg16(r) => self.regs.set_reg16(r, value as u16),
            Operand::AddrIndirect(r) => {
                let addr = self.regs.get_reg16(r);
                match r {
                    Reg16::HLplus => self.regs.set_hl(addr.wrapping_add(1)),
                    Reg16::HLminus => self.regs.set_hl(addr.wrapping_sub(1)),
                    _ => {}
                }
                bus.write8(addr, value as u8);
            }
            Operand::AddrDirect16 => bus.write8(self.read_word(bus), value as u8)
        }

    }

    fn read_byte(&mut self, bus: &MemoryBus) -> u8 {
        let val = bus.read(self.regs.pc);
        self.regs.set_pc(self.regs.get_pc().wrapping_add(1));
        val
    }

    fn read_word(&mut self, bus: &MemoryBus) -> u16 {
        let low = self.read_byte(bus) as u16;
        let high = self.read_byte(bus) as u16;
        (high << 8) | low
    }
}
