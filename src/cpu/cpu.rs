use crate::MemoryBus;
#[allow(unused_imports)]
use crate::cpu::disasm::{Condition, Instruction, Operand, Reg8, Reg16};
use crate::cpu::implements::*;
use crate::cpu::registers::*;
#[allow(unused_imports)]
use log::{debug, error, info, warn};

#[derive(Default)]
pub struct Cpu {
    pub regs: Registers,

    ime: bool,
    ime_pending: bool,
    halted: bool,
}

impl Cpu {
    pub fn new() -> Self {
        let mut new = Self::default();
        new.regs.set_af(0x01B0);
        new.regs.set_bc(0x0013);
        new.regs.set_de(0x00D8);
        new.regs.set_hl(0x014D);
        new.regs.set_sp(0xFFFE);
        new.regs.set_pc(0x100);
        new
    }

    pub fn print_state(&self) {
        let r = &self.regs;
        let f = r.get_af() as u8;
        let z = if f & 0x80 != 0 { 'Z' } else { '-' };
        let n = if f & 0x40 != 0 { 'N' } else { '-' };
        let h = if f & 0x20 != 0 { 'H' } else { '-' };
        let c = if f & 0x10 != 0 { 'C' } else { '-' };

        debug!("--- CPU State ---");
        debug!(
            "PC: {:04X}  SP: {:04X}  AF: {:04X} [{}{}{}{}]",
            r.get_pc(),
            r.get_sp(),
            r.get_af(),
            z,
            n,
            h,
            c
        );
        debug!(
            "BC: {:04X}  DE: {:04X}  HL: {:04X}",
            r.get_bc(),
            r.get_de(),
            r.get_hl()
        );
        debug!("-----------------");
    }

    pub fn print_state_doctor(&self, bus: &MemoryBus) {
        let r = &self.regs;
        let pc = r.get_pc();
        debug!(
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            r.get_a(),
            r.get_af() as u8,
            r.get_b(),
            r.get_c(),
            r.get_d(),
            r.get_e(),
            r.get_h(),
            r.get_l(),
            r.get_sp(),
            pc,
            bus.read(pc),
            bus.read(pc + 1),
            bus.read(pc + 2),
            bus.read(pc + 3)
        );
    }

    pub fn step(&mut self, mem_bus: &mut MemoryBus) -> u8 {
        self.print_state_doctor(mem_bus);
        if self.ime_pending {
            self.ime_pending = false;
            self.ime = true;
        }
        if self.handle_interrupts(mem_bus) {
            return 5;
        }
        let opcode = self.read_byte(mem_bus);
        if let Some(instr) = Instruction::decode(opcode) {
            // debug!("{:04X}: {:?}", self.regs.get_pc().wrapping_sub(1), instr);
            match instr {
                Instruction::Prefix => {
                    let next_opcode = Instruction::decode_cb(self.read_byte(mem_bus));
                    self.execute_instruction(mem_bus, &next_opcode)
                }
                _ => self.execute_instruction(mem_bus, &instr),
            }
        } else {
            error!(
                "Invalid opcode 0x{:02X} at 0x{:04X}",
                opcode,
                self.regs.get_pc().wrapping_sub(1)
            );
            0
        }
    }

    fn call(&mut self, bus: &mut MemoryBus, cond: &Option<Condition>) -> bool {
        let push_pc = self.regs.get_pc().wrapping_add(2);
        let jumped = self.jump(bus, cond, &Operand::Imm16, false);
        if !jumped {
            return false;
        }
        self.push(bus, push_pc);
        true
    }

    fn ret(&mut self, bus: &mut MemoryBus, cond: &Option<Condition>) -> bool {
        if !self.check_cond(cond) {
            return false;
        }
        let lo = bus.read(self.regs.get_sp()) as u16;
        let hi = bus.read(self.regs.get_sp().wrapping_add(1)) as u16;
        let addr = (hi << 8) | lo;
        self.regs.set_pc(addr);
        self.pop(bus);
        true
    }
    fn jump(&mut self, bus: &MemoryBus, cond: &Option<Condition>, op: &Operand, rel: bool) -> bool {
        let addr = self.get_operand_value(bus, op);
        if !self.check_cond(cond) {
            return false;
        }
        if rel {
            let offset = addr as u8 as i8;
            let new_pc = (self.regs.get_pc() as i32).wrapping_add(offset as i32) as u16;
            self.regs.set_pc(new_pc);
        } else {
            self.regs.set_pc(addr);
        }
        true
    }

    fn push(&mut self, bus: &mut MemoryBus, val: u16) {
        let sp = self.regs.get_sp().wrapping_sub(2);
        self.regs.set_sp(sp);
        bus.write16(sp, val);
    }

    fn pop(&mut self, bus: &MemoryBus) -> u16 {
        let sp = self.regs.get_sp();
        let val = bus.read(sp) as u16 | ((bus.read(sp.wrapping_add(1)) as u16) << 8);
        self.regs.set_sp(sp.wrapping_add(2));
        val
    }

    fn load(&mut self, bus: &mut MemoryBus, dst: &Operand, src: &Operand) {
        let val = self.get_operand_value(bus, src);
        self.set_operand_value(bus, dst, val);
    }

    fn apply_alu(&mut self, bus: &mut MemoryBus, dst: Option<&Operand>, res: &AluResult) {
        if let Some(z) = res.z {
            self.regs.set_flag(FLAG_Z, z);
        }
        if let Some(n) = res.n {
            self.regs.set_flag(FLAG_N, n);
        }
        if let Some(h) = res.h {
            self.regs.set_flag(FLAG_H, h);
        }
        if let Some(c) = res.c {
            self.regs.set_flag(FLAG_C, c);
        }
        if let (Some(op), Some(val)) = (dst, res.val) {
            self.set_operand_value(bus, op, val);
        }
    }

    fn execute_instruction(&mut self, bus: &mut MemoryBus, instr: &Instruction) -> u8 {
        let res: u8 = match instr {
            Instruction::NOP => 1,
            Instruction::JP(cond, op) => match (op, self.jump(bus, cond, op, false)) {
                (_, false) => 3,
                (Operand::Reg16(Reg16::HL), true) => 1,
                _ => 4,
            },
            // Instruction::LD(dst, src) | Instruction::LDH(dst, src) =>
            //     self.load(bus, dst, src),
            Instruction::LD(dst, src) => {
                self.load(bus, dst, src);
                match (dst, src) {
                    (Operand::Reg8(Reg8::A), Operand::AddrIndirect(_)) => 2,
                    (Operand::Reg8(Reg8::A), Operand::AddrDirect16) => 4,
                    (Operand::Reg8(Reg8::HLderef), Operand::Reg8(_)) => 2,
                    (Operand::Reg8(Reg8::HLderef), Operand::Imm8) => 3,
                    (Operand::Reg8(_), Operand::Reg8(Reg8::HLderef)) => 2,
                    (Operand::Reg8(_), Operand::Reg8(_)) => 1,
                    (Operand::Reg8(_), Operand::Imm8) => 2,
                    (Operand::Reg16(Reg16::SP), Operand::Reg16(Reg16::HL)) => 2,
                    (Operand::Reg16(_), Operand::Imm16) => 3,
                    (Operand::AddrIndirect(_), Operand::Reg8(Reg8::A)) => 2,
                    (Operand::AddrDirect16, Operand::Reg8(Reg8::A)) => 4,
                    // missing
                    // LD [imm16],SP
                    // LD HL,SP+imm8
                    _ => todo!(),
                }
            }
            Instruction::INC(reg) => {
                let dst_val = self.get_operand_value(bus, reg);
                match reg {
                    Operand::Reg8(Reg8::HLderef) => {
                        self.apply_alu(bus, Some(reg), &inc_u8(dst_val as u8));
                        3
                    }
                    Operand::Reg8(_) => {
                        self.apply_alu(bus, Some(reg), &inc_u8(dst_val as u8));
                        1
                    }
                    Operand::Reg16(_) => {
                        self.apply_alu(bus, Some(reg), &inc_u16(dst_val));
                        2
                    }
                    _ => unreachable!(),
                }
            }
            Instruction::DEC(reg) => {
                let dst_val = self.get_operand_value(bus, reg);
                match reg {
                    Operand::Reg8(Reg8::HLderef) => {
                        self.apply_alu(bus, Some(reg), &dec_u8(dst_val as u8));
                        3
                    }
                    Operand::Reg8(_) => {
                        self.apply_alu(bus, Some(reg), &dec_u8(dst_val as u8));
                        1
                    }
                    Operand::Reg16(_) => {
                        self.apply_alu(bus, Some(reg), &dec_u16(dst_val));
                        2
                    }
                    _ => unreachable!(),
                }
            }
            Instruction::ADD(dst, src) => {
                let dst_val = self.get_operand_value(bus, dst);
                let src_val = self.get_operand_value(bus, src);
                match (dst, src) {
                    (Operand::Reg16(Reg16::HL), Operand::Reg16(_)) => {
                        self.apply_alu(bus, Some(dst), &add_hl(dst_val, src_val));
                        2
                    }
                    (Operand::Reg16(Reg16::SP), Operand::Imm8) => {
                        self.apply_alu(bus, Some(dst), &add_sp(dst_val, src_val as u8 as i8));
                        4
                    }
                    (Operand::Reg8(Reg8::A), _) => {
                        self.apply_alu(
                            bus,
                            Some(dst),
                            &add_acc(dst_val as u8, src_val as u8, false),
                        );
                        match src {
                            Operand::Reg8(Reg8::HLderef) | Operand::Imm8 => 2,
                            Operand::Reg8(_) => 1,
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Instruction::ADC(dst, src) => match (dst, src) {
                (Operand::Reg8(Reg8::A), _) => {
                    let dst_val = self.get_operand_value(bus, dst);
                    let src_val = self.get_operand_value(bus, src);
                    self.apply_alu(
                        bus,
                        Some(dst),
                        &add_acc(dst_val as u8, src_val as u8, self.regs.get_flag(FLAG_C)),
                    );
                    match src {
                        Operand::Reg8(Reg8::HLderef) | Operand::Imm8 => 2,
                        Operand::Reg8(_) => 1,
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            },
            Instruction::SUB(dst, src) => match (dst, src) {
                (Operand::Reg8(Reg8::A), _) => {
                    let dst_val = self.get_operand_value(bus, dst);
                    let src_val = self.get_operand_value(bus, src);
                    self.apply_alu(bus, Some(dst), &sub(dst_val as u8, src_val as u8, false));
                    match src {
                        Operand::Reg8(Reg8::HLderef) | Operand::Imm8 => 2,
                        Operand::Reg8(_) => 1,
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            },
            Instruction::SBC(dst, src) => match (dst, src) {
                (Operand::Reg8(Reg8::A), _) => {
                    let dst_val = self.get_operand_value(bus, dst);
                    let src_val = self.get_operand_value(bus, src);
                    self.apply_alu(
                        bus,
                        Some(dst),
                        &sub(dst_val as u8, src_val as u8, self.regs.get_flag(FLAG_C)),
                    );
                    match src {
                        Operand::Reg8(Reg8::HLderef) | Operand::Imm8 => 2,
                        Operand::Reg8(_) => 1,
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            },
            Instruction::CP(Operand::Reg8(Reg8::A), op) => {
                let val = self.get_operand_value(bus, op) as u8;
                let res = compare(self.regs.get_a(), val);
                self.apply_alu(bus, None, &res);
                match op {
                    Operand::Reg8(Reg8::HLderef) | Operand::Imm8 => 2,
                    Operand::Reg8(_) => 1,
                    _ => unreachable!(),
                }
            }
            Instruction::AND(Operand::Reg8(Reg8::A), op) => {
                let val = self.get_operand_value(bus, op) as u8;
                let res = and(self.regs.get_a(), val);
                self.apply_alu(bus, Some(&Operand::Reg8(Reg8::A)), &res);
                match op {
                    Operand::Reg8(Reg8::HLderef) | Operand::Imm8 => 2,
                    Operand::Reg8(_) => 1,
                    _ => unreachable!(),
                }
            }
            Instruction::OR(Operand::Reg8(Reg8::A), op) => {
                let val = self.get_operand_value(bus, op) as u8;
                let res = or(self.regs.get_a(), val);
                self.apply_alu(bus, Some(&Operand::Reg8(Reg8::A)), &res);
                match op {
                    Operand::Reg8(Reg8::HLderef) | Operand::Imm8 => 2,
                    Operand::Reg8(_) => 1,
                    _ => unreachable!(),
                }
            }
            Instruction::XOR(Operand::Reg8(Reg8::A), op) => {
                let val = self.get_operand_value(bus, op) as u8;
                let res = xor(self.regs.get_a(), val);
                self.apply_alu(bus, Some(&Operand::Reg8(Reg8::A)), &res);
                match op {
                    Operand::Reg8(Reg8::HLderef) | Operand::Imm8 => 2,
                    Operand::Reg8(_) => 1,
                    _ => unreachable!(),
                }
            }
            Instruction::JR(cond, Operand::Imm8) => {
                if self.jump(bus, cond, &Operand::Imm8, true) {
                    3
                } else {
                    2
                }
            }

            Instruction::RLCA => {
                self.apply_alu(
                    bus,
                    Some(&Operand::Reg8(Reg8::A)),
                    &lrotate(self.regs.get_a(), false, None),
                );
                1
            }
            Instruction::RRCA => {
                self.apply_alu(
                    bus,
                    Some(&Operand::Reg8(Reg8::A)),
                    &rrotate(self.regs.get_a(), false, None),
                );
                1
            }
            Instruction::RLA => {
                self.apply_alu(
                    bus,
                    Some(&Operand::Reg8(Reg8::A)),
                    &lrotate(self.regs.get_a(), false, Some(self.regs.get_flag(FLAG_C))),
                );
                1
            }
            Instruction::RRA => {
                self.apply_alu(
                    bus,
                    Some(&Operand::Reg8(Reg8::A)),
                    &rrotate(self.regs.get_a(), false, Some(self.regs.get_flag(FLAG_C))),
                );
                1
            }
            Instruction::DAA => todo!("Implementing BCD instrs later"),
            Instruction::CPL => {
                self.apply_alu(
                    bus,
                    Some(&Operand::Reg8(Reg8::A)),
                    &complement(self.regs.get_a()),
                );
                1
            }
            Instruction::SCF => {
                self.apply_alu(
                    bus,
                    None,
                    &AluResult {
                        val: None,
                        z: None,
                        n: Some(false),
                        h: Some(false),
                        c: Some(true),
                    },
                );
                1
            }
            Instruction::CCF => {
                self.apply_alu(
                    bus,
                    None,
                    &AluResult {
                        val: None,
                        z: None,
                        n: Some(false),
                        h: Some(false),
                        c: Some(!self.regs.get_flag(FLAG_C)),
                    },
                );
                1
            }
            Instruction::STOP => todo!("Should implement low pow mode"),
            Instruction::HALT => todo!("Should implement low pow mode and interrupts"),

            Instruction::LDH(dst, src) => {
                self.load(bus, dst, src);
                match (dst, src) {
                    (Operand::AddrDirectLow8, Operand::Reg8(Reg8::A))
                    | (Operand::Reg8(Reg8::A), Operand::AddrDirectLow8) => 3,
                    (Operand::AddrIndirectLow8(Reg8::C), Operand::Reg8(Reg8::A))
                    | (Operand::Reg8(Reg8::A), Operand::AddrIndirectLow8(Reg8::C)) => 2,
                    _ => unreachable!(),
                }
            }

            Instruction::CALL(cond, Operand::Imm16) => {
                if self.call(bus, cond) {
                    6
                } else {
                    3
                }
            }
            Instruction::RET(cond) => {
                let res = self.ret(bus, cond);
                if cond.is_none() {
                    return 4;
                }
                if res { return 5 } else { return 2 }
            }

            Instruction::POP(r) => {
                let val = self.pop(bus);
                self.regs.set_reg16(r, val);
                3
            }

            Instruction::PUSH(r) => {
                let val = self.regs.get_reg16(r);
                self.push(bus, val);
                4
            }

            Instruction::DI => {
                self.ime = false;
                self.ime_pending = false;
                1
            }
            Instruction::EI => {
                self.ime_pending = true;
                1
            }

            Instruction::RR(Operand::Reg8(reg)) => {
                let val = self.regs.get_reg8(reg);
                self.apply_alu(
                    bus,
                    Some(&Operand::Reg8(*reg)),
                    &rrotate(val, true, Some(self.regs.get_flag(FLAG_C))),
                );
                1
            }

            Instruction::SRL(Operand::Reg8(reg)) => {
                let res = srl(self.get_operand_value(bus, &Operand::Reg8(*reg)) as u8);
                self.apply_alu(bus, Some(&Operand::Reg8(*reg)), &res);
                match reg {
                    Reg8::HLderef => 4,
                    _ => 2,
                }
            }

            Instruction::Hardlock => panic!("Hardlocked!"),
            _ => todo!(
                "{:?} (0x{:02X}) not implemented",
                instr,
                bus.read(self.regs.get_pc())
            ),
        };
        res
    }

    fn check_cond(&self, cond: &Option<Condition>) -> bool {
        match cond {
            None => true,
            Some(Condition::NonZero) => !self.regs.get_flag(FLAG_Z),
            Some(Condition::Zero) => self.regs.get_flag(FLAG_Z),
            Some(Condition::NonCarry) => !self.regs.get_flag(FLAG_C),
            Some(Condition::Carry) => self.regs.get_flag(FLAG_C),
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
            Operand::AddrDirect16 => bus.read(self.read_word(bus)) as u16,
            Operand::AddrIndirectLow8(r) => {
                assert!(*r != Reg8::HLderef);
                let addr = self.regs.get_reg8(r);
                bus.read(0xff00 | (addr as u16)) as u16
            }
            Operand::AddrDirectLow8 => {
                let addr = self.read_byte(bus);
                bus.read(0xff00 | (addr as u16)) as u16
            }
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
            Operand::Reg16(r) => self.regs.set_reg16(r, value),
            Operand::AddrIndirect(r) => {
                let addr = self.regs.get_reg16(r);
                match r {
                    Reg16::HLplus => self.regs.set_hl(addr.wrapping_add(1)),
                    Reg16::HLminus => self.regs.set_hl(addr.wrapping_sub(1)),
                    _ => {}
                }
                bus.write8(addr, value as u8);
            }
            Operand::AddrDirect16 => bus.write8(self.read_word(bus), value as u8),
            Operand::AddrIndirectLow8(r) => {
                assert!(*r != Reg8::HLderef);
                let addr = self.regs.get_reg8(r);
                bus.write8(0xff00 | (addr as u16), value as u8);
            }
            Operand::AddrDirectLow8 => {
                let addr = self.read_byte(bus);
                bus.write8(0xff00 | (addr as u16), value as u8);
            }
        }
    }

    fn handle_interrupts(&mut self, mem_bus: &mut MemoryBus) -> bool {
        let pending = mem_bus.interrupts.pending();
        if pending != 0 && self.halted {
            self.halted = false;
        }
        if !self.ime || pending == 0 {
            return false;
        }
        let bit = pending.trailing_zeros() as u16;
        self.ime = false;
        mem_bus.interrupts.if_ &= !(1 << bit);
        self.push(mem_bus, self.regs.get_pc());
        self.regs.set_pc(0x40 + bit * 8);
        true
    }

    fn read_byte(&mut self, bus: &MemoryBus) -> u8 {
        let val = bus.read(self.regs.get_pc());
        self.regs.set_pc(self.regs.get_pc().wrapping_add(1));
        val
    }

    fn read_word(&mut self, bus: &MemoryBus) -> u16 {
        let low = self.read_byte(bus) as u16;
        let high = self.read_byte(bus) as u16;
        (high << 8) | low
    }
}
