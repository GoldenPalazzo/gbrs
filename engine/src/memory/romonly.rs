use super::cartridge::Mapper;
use alloc::{boxed::Box, vec::Vec};

pub struct RomOnly {
    // rom: [u8; 0x8000],
    rom: &'static [u8],
    opt_ram: [u8; 0x2000],
}

impl Default for RomOnly {
    fn default() -> Self {
        Self {
            rom: &[],
            opt_ram: [0u8; 0x2000],
        }
    }
}

impl Mapper for RomOnly {
    fn set_rom(&mut self, rom: Vec<u8>) {
        self.rom = Box::leak(rom.into_boxed_slice());
    }
    fn set_rom_static(&mut self, rom: &'static [u8]) {
        self.rom = rom;
    }
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => self.rom[addr as usize],
            0xa000..=0xbfff => self.opt_ram[(addr as usize) - 0xa000],
            _ => unreachable!("Invalid read at 0x{:04X}", addr),
        }
    }
    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x7fff => {},
            0xa000..=0xbfff => self.opt_ram[(addr as usize) - 0xa000] = data,
            _ => unreachable!(),
        }
    }
}

