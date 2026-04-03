use crate::memory::cartridge::Mapper;

pub struct RomOnly {
    rom: [u8; 0x8000],
    opt_ram: [u8; 0x2000],
}

impl Default for RomOnly {
    fn default() -> Self {
        Self {
            rom: [0u8; 0x8000],
            opt_ram: [0u8; 0x2000],
        }
    }
}

impl Mapper for RomOnly {
    fn set_rom(&mut self, rom: Vec<u8>) {
        let len = rom.len().min(0x8000);
        self.rom.copy_from_slice(&rom[..len])
    }
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => self.rom[addr as usize],
            0xa000..=0xbfff => self.opt_ram[addr as usize],
            _ => unreachable!("Invalid read at 0x{:04X}", addr),
        }
    }
    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x7fff => println!("Write in ROM 0x{:04X} ignored", addr),
            0xa000..=0xbfff => self.opt_ram[addr as usize] = data,
            _ => unreachable!(),
        }
    }
}


