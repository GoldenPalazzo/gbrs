use super::cpu::Cpu;
use crate::memory::bus::MemoryBus;

const FLAG_Z: u8 = 0x80;
const FLAG_N: u8 = 0x40;
const FLAG_H: u8 = 0x20;
const FLAG_C: u8 = 0x10;

impl Cpu {
    pub fn execute(&mut self, bus: &mut MemoryBus, opcode: u8) -> u8 {
        match opcode {
            0x00 => 1,
            0xc2 | 0xca | 0xd2 | 0xda => { // JP cc, imm16
                let addr = self.read_word(bus);
                if self.check_cond_bits(opcode) {
                    self.regs.set_pc(addr);
                    return 4
                }
                3
            }
            0xc3 => { // JP imm16
                let addr = self.read_word(bus);
                self.regs.set_pc(addr);
                4
            }
            0xe9 => { // JP HL
                self.regs.set_pc(self.regs.get_hl());
                1
            }
            0x18 => { // JR imm8
                let off = self.read_byte(bus) as i8;
                let res = (self.regs.get_pc() as i16).wrapping_add(off as i16);
                self.regs.set_pc(res as u16);
                3
            }
            0x20 | 0x28 | 0x30 | 0x38 => { // JR cc, imm8
                let off = self.read_byte(bus) as i8;
                if self.check_cond_bits(opcode) {
                    let res = (self.regs.get_pc() as i16).wrapping_add(off as i16);
                    self.regs.set_pc(res as u16);
                    return 3
                }
                2
            }
            0x10 => {
                todo!("STOP")
            }
            0x76 => {
                let pending = bus.interrupts.pending() != 0;
                if !self.ime && pending {
                    self.halt_bug = true;
                } else {
                    self.halted = true;
                }
                1
            }
            0x01 | 0x11 | 0x21 | 0x31 => {  // LD r16, imm16
                let val = self.read_word(bus);
                self.write_r16_normal(opcode, val);
                3
            }
            0x02 | 0x12 | 0x22 | 0x32 => {  // LD [r16mem], A
                let addr = self.read_r16_mem(opcode);
                bus.write8(addr, self.regs.get_a());
                2
            }
            0x0a | 0x1a | 0x2a | 0x3a => {  // LD A, [r16mem]
                let addr = self.read_r16_mem(opcode);
                let val = bus.read(addr);
                self.regs.set_a(val);
                2
            }
            0x08 => {  // LD [imm16], SP
                let addr = self.read_word(bus);
                let sp = self.regs.get_sp();
                bus.write16(addr, sp);
                5 
            }
            0x03 | 0x13 | 0x23 | 0x33 => { // INC r16
                let val = self.read_r16_normal(opcode);
                self.write_r16_normal(opcode, val.wrapping_add(1));
                2
            }
            0x0b | 0x1b | 0x2b | 0x3b => { // DEC r16
                let val = self.read_r16_normal(opcode);
                self.write_r16_normal(opcode, val.wrapping_sub(1));
                2
            }
            0x09 | 0x19 | 0x29 | 0x39 => { // ADD HL, r16
                let val = self.read_r16_normal(opcode);
                let hl = self.regs.get_hl();
                let result = hl.wrapping_add(val);
                self.regs.set_hl(result);
                self.regs.set_flag(FLAG_N, false);
                self.regs.set_flag(FLAG_H, (hl & 0xfff) + (val & 0xfff) > 0xfff);
                self.regs.set_flag(FLAG_C, result < hl);
                2
            }
            0x04 | 0x0c | 0x14 | 0x1c | 0x24 | 0x2c | 0x34 | 0x3c => { // INC r8
                let val = self.read_r8(bus, opcode, 3);
                let result = val.wrapping_add(1);
                self.write_r8(bus, opcode, 3, result);
                self.regs.set_flag(FLAG_Z, result == 0);
                self.regs.set_flag(FLAG_N, false);
                self.regs.set_flag(FLAG_H, (val & 0xf) == 0xf);
                if (opcode >> 3) & 7 == 6 { 3 } else { 1 }
            }
            0x05 | 0x0d | 0x15 | 0x1d | 0x25 | 0x2d | 0x35 | 0x3d => { // DEC r8
                let val = self.read_r8(bus, opcode, 3);
                let result = val.wrapping_sub(1);
                self.write_r8(bus, opcode, 3, result);
                self.regs.set_flag(FLAG_Z, result == 0);
                self.regs.set_flag(FLAG_N, true);
                self.regs.set_flag(FLAG_H, (val & 0xf) == 0);
                if (opcode >> 3) & 7 == 6 { 3 } else { 1 }
            }
            0x06 | 0x0e | 0x16 | 0x1e | 0x26 | 0x2e | 0x36 | 0x3e => { // LD r8, imm8
                let val = self.read_byte(bus);
                self.write_r8(bus, opcode, 3, val);
                if (opcode >> 3) & 7 == 6 { 3 } else { 2 }
            }
            0x07 => { // RLCA
                let res = self.alu_rlc(self.regs.get_a());
                self.regs.set_a(res);
                self.regs.set_flag(FLAG_Z, false);
                1
            }
            0x0f => { // RRCA
                let res = self.alu_rrc(self.regs.get_a());
                self.regs.set_a(res);
                self.regs.set_flag(FLAG_Z, false);
                1
            }
            0x17 => { // RLA
                let res = self.alu_rl(self.regs.get_a());
                self.regs.set_a(res);
                self.regs.set_flag(FLAG_Z, false);
                1
            }
            0x1f => { // RRA
                let res = self.alu_rr(self.regs.get_a());
                self.regs.set_a(res);
                self.regs.set_flag(FLAG_Z, false);
                1
            }
            0x27 => { // DAA
                self.alu_daa();
                1
            }
            0x2f => { // CPL
                self.regs.set_a(!self.regs.get_a());
                self.regs.set_flag(FLAG_N, true);
                self.regs.set_flag(FLAG_H, true);
                1
            }
            0x37 => { // SCF 
                self.regs.set_flag(FLAG_N, false);
                self.regs.set_flag(FLAG_H, false);
                self.regs.set_flag(FLAG_C, true);
                1
            }
            0x3f => { // CCF 
                self.regs.set_flag(FLAG_N, false);
                self.regs.set_flag(FLAG_H, false);
                self.regs.set_flag(FLAG_C, !self.regs.get_flag(FLAG_C));
                1
            }

            0x40..=0x7F => { // LD r8, r8
                if opcode == 0x76 { // HALT
                    unreachable!();
                }
                let src = self.read_r8(bus, opcode, 0);
                self.write_r8(bus, opcode, 3, src);
                if (opcode & 0x07 == 6) || ((opcode >> 3) & 0x07 == 6) { 2 } else { 1 }
            }

            // --- Block 2 ---
            0x80..=0x87 => { // ADD A, r8
                let val = self.read_r8(bus, opcode, 0);
                self.alu_add(val, false);
                if opcode & 7 == 6 { 2 } else { 1 }
            }
            0x88..=0x8f => { // ADC A, r8
                let val = self.read_r8(bus, opcode, 0);
                self.alu_add(val, self.regs.get_flag(FLAG_C));
                if opcode & 7 == 6 { 2 } else { 1 }
            }
            0x90..=0x97 => { // SUB A, r8
                let val = self.read_r8(bus, opcode, 0);
                self.alu_sub(val, false);
                if opcode & 7 == 6 { 2 } else { 1 }
            }
            0x98..=0x9f => { // SBC A, r8
                let val = self.read_r8(bus, opcode, 0);
                self.alu_sub(val, self.regs.get_flag(FLAG_C));
                if opcode & 7 == 6 { 2 } else { 1 }
            }
            0xa0..=0xa7 => { // AND A, r8
                let val = self.read_r8(bus, opcode, 0);
                self.alu_and(val);
                if opcode & 7 == 6 { 2 } else { 1 }
            }
            0xa8..=0xaf => { // XOR A, r8
                let val = self.read_r8(bus, opcode, 0);
                self.alu_xor(val);
                if opcode & 7 == 6 { 2 } else { 1 }
            }
            0xb0..=0xb7 => { // OR A, r8
                let val = self.read_r8(bus, opcode, 0);
                self.alu_or(val);
                if opcode & 7 == 6 { 2 } else { 1 }
            }
            0xb8..=0xbf => { // CP A, r8
                let val = self.read_r8(bus, opcode, 0);
                self.alu_cp(val);
                if opcode & 7 == 6 { 2 } else { 1 }
            }
            
            // --- Block 3 ---
            0xc6 => { // ADD A, imm8
                let val = self.read_byte(bus);
                self.alu_add(val, false);
                2
            }
            0xce => { // ADC A, imm8
                let val = self.read_byte(bus);
                self.alu_add(val, self.regs.get_flag(FLAG_C));
                2
            }
            0xd6 => { // SUB A, imm8
                let val = self.read_byte(bus);
                self.alu_sub(val, false);
                2
            }
            0xde => { // SBC A, imm8
                let val = self.read_byte(bus);
                self.alu_sub(val, self.regs.get_flag(FLAG_C));
                2
            }
            0xe6 => { // AND A, imm8
                let val = self.read_byte(bus);
                self.alu_and(val);
                2
            }
            0xee => { // XOR A, imm8
                let val = self.read_byte(bus);
                self.alu_xor(val);
                2
            }
            0xf6 => { // OR A, imm8
                let val = self.read_byte(bus);
                self.alu_or(val);
                2
            }
            0xfe => { // CP A, imm8
                let val = self.read_byte(bus);
                self.alu_cp(val);
                2
            }
            0xc0 | 0xc8 | 0xd0 | 0xd8 => { // RET cond
                if self.check_cond_bits(opcode) {
                    let addr = self._pop(bus);
                    self.regs.set_pc(addr);
                    return 5;
                }
                2
            }
            0xc9 => { // RET
                let addr = self._pop(bus);
                self.regs.set_pc(addr);
                4
            }
            0xd9 => { // RETI
                let addr = self._pop(bus);
                self.regs.set_pc(addr);
                self.ime_pending = true;
                4
            }
            0xc4 | 0xcc | 0xd4 | 0xdc => { // CALL cond, imm16
                let addr = self.read_word(bus);
                if self.check_cond_bits(opcode) {
                    self._push(bus, self.regs.get_pc());
                    self.regs.set_pc(addr);
                    return 6;
                }
               3 
            }
            0xcd => { // CALL imm16
                let addr = self.read_word(bus);
                self._push(bus, self.regs.get_pc());
                self.regs.set_pc(addr);
               6 
            }
            0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => { // RST tgt3
                let tgt = (opcode & 0x38) as u16;
                self._push(bus, self.regs.get_pc());
                self.regs.set_pc(tgt);
                4
            }
            0xc1 | 0xd1 | 0xe1 | 0xf1 => { // POP r16stk
                let addr = self._pop(bus);
                self.write_r16_stk(opcode, addr);
                3
            }
            0xc5 | 0xd5 | 0xe5 | 0xf5 => { // PUSH r16stk
                let addr = self.read_r16_stk(opcode);
                self._push(bus, addr);
                4
            }
            0xcb => {
                let cb_opcode = self.read_byte(bus);
                self.execute_cb(bus, cb_opcode)
            }
            0xe2 => { // LDH [c], a
                let a = self.regs.get_a();
                bus.write8(0xff00u16.wrapping_add(self.regs.get_c() as u16), a);
                2
            }
            0xe0 => { // LDH [imm8], a
                let a = self.regs.get_a();
                let addr = self.read_byte(bus) as u16;
                bus.write8(0xff00u16.wrapping_add(addr), a);
                3 
            }
            0xea => { // LD [imm16], a
                let a = self.regs.get_a();
                let addr = self.read_word(bus);
                bus.write8(addr, a);
                4
            }
            0xf2 => { // LDH a, [c]
                let addr = 0xff00u16.wrapping_add(self.regs.get_c() as u16);
                let val = bus.read(addr);
                self.regs.set_a(val);
                2
            }
            0xf0 => { // LDH a, [imm8]
                let addr = 0xff00u16.wrapping_add(self.read_byte(bus) as u16);
                let val = bus.read(addr);
                self.regs.set_a(val);
                3 
            }
            0xfa => { // LD a, [imm16]
                let addr = self.read_word(bus);
                let val = bus.read(addr);
                self.regs.set_a(val);
                4
            }
            0xe8 => { // ADD SP, imm8 (signed)
                let val = self.read_byte(bus);
                let signed = val as i8 as i32;
                let sp = self.regs.get_sp();
                let result = (sp as i32).wrapping_add(signed) as u16;
                self.regs.set_sp(result);
                self.regs.set_flag(FLAG_Z, false);
                self.regs.set_flag(FLAG_N, false);
                self.regs.set_flag(FLAG_H, (sp & 0xf) + (val as u16 & 0xf) > 0xf);
                self.regs.set_flag(FLAG_C, (sp & 0xff) + val as u16 > 0xff);
                4
            }
            0xf8 => { // LD HL, SP + imm8
                let val = self.read_byte(bus);
                let signed = val as i8 as i32;
                let sp = self.regs.get_sp();
                let result = (sp as i32).wrapping_add(signed) as u16;
                self.regs.set_hl(result);
                self.regs.set_flag(FLAG_Z, false);
                self.regs.set_flag(FLAG_N, false);
                self.regs.set_flag(FLAG_H, (sp & 0xf) + (val as u16 & 0xf) > 0xf);
                self.regs.set_flag(FLAG_C, (sp & 0xff) + val as u16 > 0xff);
                3
            }
            0xf9 => { // LD SP, HL
                self.regs.set_sp(self.regs.get_hl());
                2
            }
            0xf3 => { // DI
                self.ime = false;
                self.ime_pending = false;
                1
            }
            0xfb => { // EI
                self.ime_pending = true;
                1
            }
            0xd3 | 0xdb | 0xdd | 0xe3 | 0xe4 | 0xeb | 0xec | 0xed | 0xf4 | 0xfc | 0xfd => {
                todo!("Hardlock")
            }
            // _ => 0,
        }
    }

    pub fn execute_cb(&mut self, bus: &mut MemoryBus, opcode: u8) -> u8 {
        let r = opcode & 0x07;
        let bit = (opcode >> 3) & 0x07;
        
        match opcode >> 6 {
            0 => match bit {
                0 => { // RLC r8
                    let val = self.read_r8(bus, opcode, 0);
                    let res = self.alu_rlc(val);
                    self.write_r8(bus, opcode, 0, res);
                    if r == 6 { 4 } else { 2 }
                }
                1 => { // RRC r8
                    let val = self.read_r8(bus, opcode, 0);
                    let res = self.alu_rrc(val);
                    self.write_r8(bus, opcode, 0, res);
                    if r == 6 { 4 } else { 2 }
                }
                2 => { // RL r8
                    let val = self.read_r8(bus, opcode, 0);
                    let res = self.alu_rl(val);
                    self.write_r8(bus, opcode, 0, res);
                    if r == 6 { 4 } else { 2 }
                }
                3 => { // RR r8
                    let val = self.read_r8(bus, opcode, 0);
                    let res = self.alu_rr(val);
                    self.write_r8(bus, opcode, 0, res);
                    if r == 6 { 4 } else { 2 }
                }
                4 => { // SLA r8
                    let val = self.read_r8(bus, opcode, 0);
                    let res = val << 1;
                    self.regs.set_flag(FLAG_C, val & 0x80 != 0);
                    self.regs.set_flag(FLAG_Z, res == 0);
                    self.regs.set_flag(FLAG_N, false);
                    self.regs.set_flag(FLAG_H, false);
                    self.write_r8(bus, opcode, 0, res);
                    if r == 6 { 4 } else { 2 }
                }
                5 => { // SRA r8
                    let val = self.read_r8(bus, opcode, 0);
                    let res = (val >> 1) | (val & 0x80); // preserva bit 7
                    self.regs.set_flag(FLAG_C, val & 1 != 0);
                    self.regs.set_flag(FLAG_Z, res == 0);
                    self.regs.set_flag(FLAG_N, false);
                    self.regs.set_flag(FLAG_H, false);
                    self.write_r8(bus, opcode, 0, res);
                    if r == 6 { 4 } else { 2 }
                }
                6 => { // SWAP r8
                    let val = self.read_r8(bus, opcode, 0);
                    let res = val.rotate_left(4);
                    self.regs.set_flag(FLAG_Z, res == 0);
                    self.regs.set_flag(FLAG_N, false);
                    self.regs.set_flag(FLAG_H, false);
                    self.regs.set_flag(FLAG_C, false);
                    self.write_r8(bus, opcode, 0, res);
                    if r == 6 { 4 } else { 2 }
                }
                7 => { // SRL r8
                    let val = self.read_r8(bus, opcode, 0);
                    let res = val >> 1;
                    self.regs.set_flag(FLAG_C, val & 1 != 0);
                    self.regs.set_flag(FLAG_Z, res == 0);
                    self.regs.set_flag(FLAG_N, false);
                    self.regs.set_flag(FLAG_H, false);
                    self.write_r8(bus, opcode, 0, res);
                    if r == 6 { 4 } else { 2 }
                }
                _ => unreachable!()
            }
            1 => { // BIT
                let val = self.read_r8(bus, opcode, 0);
                self.regs.set_flag(FLAG_Z, val & (1 << bit) == 0);
                self.regs.set_flag(FLAG_N, false);
                self.regs.set_flag(FLAG_H, true);
                if r == 6 { 3 } else { 2 }
            }
            2 => { // RES
                let val = self.read_r8(bus, opcode, 0);
                self.write_r8(bus, opcode, 0, val & !(1 << bit));
                if r == 6 { 4 } else { 2 }
            }
            3 => { // SET
                let val = self.read_r8(bus, opcode, 0);
                self.write_r8(bus, opcode, 0, val | (1 << bit));
                if r == 6 { 4 } else { 2 }
            }
            _ => unreachable!()
        }
    }

    #[inline(always)]
    fn read_r8(&self, bus: &MemoryBus, opcode: u8, lsb: u8) -> u8 {
        match (opcode >> lsb) & 7 {
            0 => self.regs.get_b(),
            1 => self.regs.get_c(),
            2 => self.regs.get_d(),
            3 => self.regs.get_e(),
            4 => self.regs.get_h(),
            5 => self.regs.get_l(),
            6 => bus.read(self.regs.get_hl()),
            7 => self.regs.get_a(),
            _ => unreachable!("Reg8 extraction failed"),
        }
    }

    #[inline(always)]
    fn write_r8(&mut self, bus: &mut MemoryBus, opcode: u8, lsb: u8, value: u8) {
        match (opcode >> lsb) & 7 {
            0 => self.regs.set_b(value),
            1 => self.regs.set_c(value),
            2 => self.regs.set_d(value),
            3 => self.regs.set_e(value),
            4 => self.regs.set_h(value),
            5 => self.regs.set_l(value),
            6 => bus.write8(self.regs.get_hl(), value),
            7 => self.regs.set_a(value),
            _ => unreachable!("Reg8 extraction failed"),
        }
    }

    #[inline(always)]
    fn read_r16_normal(&self, opcode: u8) -> u16 {
        match (opcode >> 4) & 3 {
            0 => self.regs.get_bc(),
            1 => self.regs.get_de(),
            2 => self.regs.get_hl(),
            3 => self.regs.get_sp(),
            _ => unreachable!("Reg8 extraction failed"),
        }
    }

    #[inline(always)]
    fn read_r16_stk(&self, opcode: u8) -> u16 {
        match (opcode >> 4) & 3 {
            0 => self.regs.get_bc(),
            1 => self.regs.get_de(),
            2 => self.regs.get_hl(),
            3 => self.regs.get_af(),
            _ => unreachable!()
        }
    }

    #[inline(always)]
    fn read_r16_mem(&mut self, opcode: u8) -> u16 {
        match (opcode >> 4) & 3 {
            0 => self.regs.get_bc(),
            1 => self.regs.get_de(),
            2 => { let v = self.regs.get_hl(); self.regs.set_hl(v.wrapping_add(1)); v }
            3 => { let v = self.regs.get_hl(); self.regs.set_hl(v.wrapping_sub(1)); v }
            _ => unreachable!()
        }
    }

    #[inline(always)]
    fn write_r16_normal(&mut self, opcode: u8, val: u16) {
        match (opcode >> 4) & 3 {
            0 => self.regs.set_bc(val),
            1 => self.regs.set_de(val),
            2 => self.regs.set_hl(val),
            3 => self.regs.set_sp(val),
            _ => unreachable!()
        }
    }

    #[inline(always)]
    fn write_r16_stk(&mut self, opcode: u8, val: u16) {
        match (opcode >> 4) & 3 {
            0 => self.regs.set_bc(val),
            1 => self.regs.set_de(val),
            2 => self.regs.set_hl(val),
            3 => self.regs.set_af(val & 0xFFF0), // i 4 bit bassi di F sono sempre 0
            _ => unreachable!()
        }
    }

    #[inline(always)]
    fn check_cond_bits(&self, opcode: u8) -> bool {
        match (opcode >> 3) & 3 {
            0 => !self.regs.get_flag(FLAG_Z), // NonZero
            1 => self.regs.get_flag(FLAG_Z),  // Zero
            2 => !self.regs.get_flag(FLAG_C), // NonCarry
            3 => self.regs.get_flag(FLAG_C),  // Carry
            _ => unreachable!()
        }
    }

    #[inline(always)]
    fn alu_add(&mut self, val: u8, carry: bool) {
        let a = self.regs.get_a();
        let c = carry as u8;
        let result = a.wrapping_add(val).wrapping_add(c);
        self.regs.set_a(result);
        self.regs.set_flag(FLAG_Z, result == 0);
        self.regs.set_flag(FLAG_N, false);
        self.regs.set_flag(FLAG_H, (a & 0xf) + (val & 0xf) + c > 0xf);
        self.regs.set_flag(FLAG_C, (a as u16 + val as u16 + c as u16) > 0xff);
    }

    #[inline(always)]
    fn alu_sub(&mut self, val: u8, carry: bool) {
        let a = self.regs.get_a();
        let c = carry as u8;
        let result = a.wrapping_sub(val).wrapping_sub(c);
        self.regs.set_a(result);
        self.regs.set_flag(FLAG_Z, result == 0);
        self.regs.set_flag(FLAG_N, true);
        self.regs.set_flag(FLAG_H, (a & 0xf) < (val & 0xf) + c);
        self.regs.set_flag(FLAG_C, val as u16 + c as u16 > a as u16);
    }

    #[inline(always)]
    fn alu_and(&mut self, val: u8) {
        let a = self.regs.get_a();
        let result = a & val;
        self.regs.set_a(result);
        self.regs.set_flag(FLAG_Z, result == 0);
        self.regs.set_flag(FLAG_N, false);
        self.regs.set_flag(FLAG_H, true);
        self.regs.set_flag(FLAG_C, false);
    }

    #[inline(always)]
    fn alu_xor(&mut self, val: u8) {
        let a = self.regs.get_a();
        let result = a ^ val;
        self.regs.set_a(result);
        self.regs.set_flag(FLAG_Z, result == 0);
        self.regs.set_flag(FLAG_N, false);
        self.regs.set_flag(FLAG_H, false);
        self.regs.set_flag(FLAG_C, false);
    }

    #[inline(always)]
    fn alu_or(&mut self, val: u8) {
        let a = self.regs.get_a();
        let result = a | val;
        self.regs.set_a(result);
        self.regs.set_flag(FLAG_Z, result == 0);
        self.regs.set_flag(FLAG_N, false);
        self.regs.set_flag(FLAG_H, false);
        self.regs.set_flag(FLAG_C, false);
    }

    #[inline(always)]
    fn alu_cp(&mut self, val: u8) {
        let a = self.regs.get_a();
        self.regs.set_flag(FLAG_Z, a.wrapping_sub(val) == 0);
        self.regs.set_flag(FLAG_N, true);
        self.regs.set_flag(FLAG_H, (a & 0xf) < (val & 0xf));
        self.regs.set_flag(FLAG_C, val > a);
    }

    #[inline(always)]
    pub fn push(&mut self, bus: &mut MemoryBus, val: u16) {
        let sp = self.regs.get_sp().wrapping_sub(2);
        self.regs.set_sp(sp);
        bus.write16(sp, val);
    }

    #[inline(always)]
    pub fn pop(&mut self, bus: &MemoryBus) -> u16 {
        let sp = self.regs.get_sp();
        let val = bus.read(sp) as u16 | ((bus.read(sp.wrapping_add(1)) as u16) << 8);
        self.regs.set_sp(sp.wrapping_add(2));
        val
    }

    #[inline(always)]
    fn alu_rlc(&mut self, val: u8) -> u8 {
        let res = val.rotate_left(1);
        self.regs.set_flag(FLAG_Z, res == 0);
        self.regs.set_flag(FLAG_N, false);
        self.regs.set_flag(FLAG_H, false);
        self.regs.set_flag(FLAG_C, val & 0x80 != 0);
        res
    }

    #[inline(always)]
    fn alu_rl(&mut self, val: u8) -> u8 {
        let old_carry = self.regs.get_flag(FLAG_C) as u8;
        let res = (val << 1) | old_carry;
        self.regs.set_flag(FLAG_Z, res == 0);
        self.regs.set_flag(FLAG_N, false);
        self.regs.set_flag(FLAG_H, false);
        self.regs.set_flag(FLAG_C, val & 0x80 != 0);
        res
    }

    #[inline(always)]
    fn alu_rrc(&mut self, val: u8) -> u8 {
        let res = val.rotate_right(1);
        self.regs.set_flag(FLAG_Z, res == 0);
        self.regs.set_flag(FLAG_N, false);
        self.regs.set_flag(FLAG_H, false);
        self.regs.set_flag(FLAG_C, val & 1 != 0);
        res
    }

    #[inline(always)]
    fn alu_rr(&mut self, val: u8) -> u8 {
        let old_carry = self.regs.get_flag(FLAG_C) as u8;
        let res = (val >> 1) | (old_carry << 7);
        self.regs.set_flag(FLAG_Z, res == 0);
        self.regs.set_flag(FLAG_N, false);
        self.regs.set_flag(FLAG_H, false);
        self.regs.set_flag(FLAG_C, val & 1 != 0);
        res
    }

    #[inline(always)]
    fn alu_daa(&mut self) {
        let a = self.regs.get_a();
        let mut res = self.regs.get_a();
        let n_flag =self.regs.get_flag(FLAG_N);
        let c_flag =self.regs.get_flag(FLAG_C);
        let h_flag =self.regs.get_flag(FLAG_H);
        match n_flag {
            true => {
                let mut adj = 0;
                if h_flag {
                    adj += 6;
                }
                if c_flag {
                    adj += 0x60;
                }
                res = res.wrapping_sub(adj);
            }
            false => {
                let mut adj = 0;
                if h_flag || (a & 0xf) > 0x9 {
                    adj += 6;
                }
                if c_flag || a > 0x99 {
                    adj += 0x60;
                    self.regs.set_flag(FLAG_C, true);
                }
                res = res.wrapping_add(adj);
            }
        };
        self.regs.set_a(res);
        self.regs.set_flag(FLAG_Z, res == 0);
        self.regs.set_flag(FLAG_H, false);
    }
}


