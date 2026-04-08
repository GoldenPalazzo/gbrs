#[allow(unused_imports)]
use super::disasm::{Condition, Instruction, Operand, Reg8, Reg16};
use super::execute::*;
use super::implements::*;
use super::registers::*;
use crate::memory::bus::MemoryBus;
#[allow(unused_imports)]
// use log::{debug, error, info, warn};
#[derive(Default)]
pub struct Cpu {
    pub regs: Registers,

    pub ime: bool,
    pub ime_pending: bool,
    pub halted: bool,
    pub halt_bug: bool,
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

    #[cfg(feature = "std")]
    pub fn print_state(&self) {
        let r = &self.regs;
        let f = r.get_af() as u8;
        let z = if f & 0x80 != 0 { 'Z' } else { '-' };
        let n = if f & 0x40 != 0 { 'N' } else { '-' };
        let h = if f & 0x20 != 0 { 'H' } else { '-' };
        let c = if f & 0x10 != 0 { 'C' } else { '-' };

        // debug!("--- CPU State ---");
        // debug!(
        //     "PC: {:04X}  SP: {:04X}  AF: {:04X} [{}{}{}{}]",
        //     r.get_pc(),
        //     r.get_sp(),
        //     r.get_af(),
        //     z,
        //     n,
        //     h,
        //     c
        // );
        // debug!(
        //     "BC: {:04X}  DE: {:04X}  HL: {:04X}",
        //     r.get_bc(),
        //     r.get_de(),
        //     r.get_hl()
        // );
        // debug!("-----------------");
    }

    #[cfg(feature = "std")]
    pub fn print_state_doctor(&self, bus: &MemoryBus) {
        let r = &self.regs;
        let pc = r.get_pc();
        // debug!(
        //     "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
        //     r.get_a(),
        //     r.get_af() as u8,
        //     r.get_b(),
        //     r.get_c(),
        //     r.get_d(),
        //     r.get_e(),
        //     r.get_h(),
        //     r.get_l(),
        //     r.get_sp(),
        //     pc,
        //     bus.read(pc),
        //     bus.read(pc + 1),
        //     bus.read(pc + 2),
        //     bus.read(pc + 3)
        // );
    }

    pub fn step(&mut self, mem_bus: &mut MemoryBus) -> u8 {
        if self.ime_pending {
            self.ime_pending = false;
            self.ime = true;
        }

        if self.halted {
            let pending = mem_bus.interrupts.pending() != 0;
            if pending {
                self.halted = false;
            }
            return 1;
        }
        if self.handle_interrupts(mem_bus) {
            return 5;
        }
        let opcode = self.read_byte(mem_bus);
        self.execute(mem_bus, opcode)
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

    pub fn read_byte(&mut self, bus: &MemoryBus) -> u8 {
        let val = bus.read(self.regs.get_pc());
        if self.halt_bug {
            self.halt_bug = false;
        } else {
            self.regs.set_pc(self.regs.get_pc().wrapping_add(1));
        }
        val
    }

    pub fn read_word(&mut self, bus: &MemoryBus) -> u16 {
        let low = self.read_byte(bus) as u16;
        let high = self.read_byte(bus) as u16;
        (high << 8) | low
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::bus::MemoryBus;

    fn make_cpu_mem(rom: &[u8]) -> (Cpu, MemoryBus) {
        let mut cpu = Cpu::new();
        let mut mem = MemoryBus::default();
        // scrivi i byte direttamente nella HRAM per evitare il mapper
        for (i, &b) in rom.iter().enumerate() {
            mem.write8(0xff80 + i as u16, b);
        }
        cpu.regs.set_pc(0xff80);
        (cpu, mem)
    }

    #[test]
    fn test_ld_hl_sp_plus_e8() {
        let (mut cpu, mut mem) = make_cpu_mem(&[0xF8, 0x01]); // LD HL, SP+1
        cpu.regs.set_sp(0x0000);
        cpu.regs.set_hl(0x0000);
        cpu.step(&mut mem);
        assert_eq!(cpu.regs.get_hl(), 0x0001);
        // flag H e C devono essere 0 in questo caso
        assert!(!cpu.regs.get_flag(FLAG_H));
        assert!(!cpu.regs.get_flag(FLAG_C));
    }

    #[test]
    fn test_ld_hl_sp_plus_e8_negative() {
        let (mut cpu, mut mem) = make_cpu_mem(&[0xF8, 0xFF]); // LD HL, SP+(-1)
        cpu.regs.set_sp(0x0002);
        cpu.step(&mut mem);
        assert_eq!(cpu.regs.get_hl(), 0x0001);
    }
}
