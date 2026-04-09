use super::{mbc1::Mbc1, mbc3::Mbc3, romonly::RomOnly};
use alloc::vec::Vec;

// pub trait Mapper {
//     fn set_rom(&mut self, rom: Vec<u8>);
//     fn set_rom_static(&mut self, rom: &'static [u8]);
//     fn read(&self, addr: u16) -> u8;
//     fn write(&mut self, addr: u16, data: u8);
// }

pub enum Mapper {
    RomOnly(RomOnly),
    Mbc1(Mbc1),
    Mbc3(Mbc3),
}

impl Mapper {
    fn set_rom(&mut self, rom: Vec<u8>) {
        match self {
            Mapper::RomOnly(m) => m.set_rom(rom),
            Mapper::Mbc1(m) => m.set_rom(rom),
            Mapper::Mbc3(m) => m.set_rom(rom),
        }
    }

    fn set_rom_static(&mut self, rom: &'static [u8]) {
        match self {
            Mapper::RomOnly(m) => m.set_rom_static(rom),
            Mapper::Mbc1(m) => m.set_rom_static(rom),
            Mapper::Mbc3(m) => m.set_rom_static(rom),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match self {
            Mapper::RomOnly(m) => m.read(addr),
            Mapper::Mbc1(m) => m.read(addr),
            Mapper::Mbc3(m) => m.read(addr),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match self {
            Mapper::RomOnly(m) => m.write(addr, data),
            Mapper::Mbc1(m) => m.write(addr, data),
            Mapper::Mbc3(m) => m.write(addr, data),
        }
    }
}

pub struct Cartridge {
    // pub title: String,
    pub mapper: Mapper,
}

impl Default for Cartridge {
    fn default() -> Self {
        Self {
            // title: String::default(),
            mapper: Mapper::RomOnly(RomOnly::default()),
        }
    }
}

impl Cartridge {
    #[cfg(feature = "std")]
    pub fn from_file(path: &str) -> std::io::Result<Self> {
        let data = std::fs::read(path)?;
        let hw_type = data[0x147];
        // let title = String::from_utf8_lossy(&data[0x134..0x144]).to_string();
        let mut mapper: Mapper = match hw_type {
            0x00 => Mapper::RomOnly(RomOnly::default()),
            0x01 => Mapper::Mbc1(Mbc1::new(false, false)),
            0x02 => Mapper::Mbc1(Mbc1::new(true, false)),
            0x03 => Mapper::Mbc1(Mbc1::new(true, true)),
            0x0f => Mapper::Mbc3(Mbc3::new(false, true, true)),
            0x10 => Mapper::Mbc3(Mbc3::new(true, true, true)),
            0x11 => Mapper::Mbc3(Mbc3::new(false, false, false)),
            0x12 => Mapper::Mbc3(Mbc3::new(true, false, false)),
            0x13 => Mapper::Mbc3(Mbc3::new(true, false, true)),
            _ => todo!("Mapper {} not implemented", hw_type),
        };
        mapper.set_rom(data);
        // Ok(Self { title, mapper })
        Ok(Self { mapper })
    }

    pub fn from_static(data: &'static [u8]) -> Self {
        let hw_type = data[0x147];
        // let title = String::from_utf8_lossy(&data[0x134..0x144]).to_string();
        let mut mapper: Mapper = match hw_type {
            0x00 => Mapper::RomOnly(RomOnly::default()),
            0x01 => Mapper::Mbc1(Mbc1::new(false, false)),
            0x02 => Mapper::Mbc1(Mbc1::new(true, false)),
            0x03 => Mapper::Mbc1(Mbc1::new(true, true)),
            0x0f => Mapper::Mbc3(Mbc3::new(false, true, true)),
            0x10 => Mapper::Mbc3(Mbc3::new(true, true, true)),
            0x11 => Mapper::Mbc3(Mbc3::new(false, false, false)),
            0x12 => Mapper::Mbc3(Mbc3::new(true, false, false)),
            0x13 => Mapper::Mbc3(Mbc3::new(true, false, true)),
            _ => todo!("Mapper {} not implemented", hw_type),
        };
        mapper.set_rom_static(data);
        // Ok(Self { title, mapper })
        Self { mapper }
    }
}
