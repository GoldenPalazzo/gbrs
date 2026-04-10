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
    fn dirty_sram(&self) -> bool {
        match self {
            Mapper::RomOnly(_) => false,
            Mapper::Mbc1(m) => m.has_battery && m.dirty_ram,
            Mapper::Mbc3(m) => m.has_battery && m.dirty_ram,
        }
    }

    fn clear_dirty(&mut self) {
        match self {
            Mapper::RomOnly(_) => {}
            Mapper::Mbc1(m) => m.dirty_ram = false,
            Mapper::Mbc3(m) => m.dirty_ram = false,
        }
    }

    pub fn ram_slice(&self) -> Option<&[u8]> {
        match self {
            Mapper::Mbc1(m) if m.has_battery => Some(&m.ram),
            Mapper::Mbc3(m) if m.has_battery => Some(&m.ram),
            _ => None,
        }
    }

    pub fn ram_slice_mut(&mut self) -> Option<&mut [u8]> {
        match self {
            Mapper::Mbc1(m) if m.has_battery => Some(&mut m.ram),
            Mapper::Mbc3(m) if m.has_battery => Some(&mut m.ram),
            _ => None,
        }
    }

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
    pub fn from_file(path: &std::path::Path) -> std::io::Result<Self> {
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
        match std::fs::read(path.with_extension("gbsav")) {
            Ok(saved) => {
                if let Some(sram) = mapper.ram_slice_mut() {
                    std::println!("Loaded save {:?}", path.with_extension("gbsav"));
                    let len = sram.len().min(saved.len());
                    sram[..len].copy_from_slice(&saved[..len]);
                }
            }
            Err(_) => {
                std::println!("No save {:?}", path.with_extension("gbsav"));
            }
        }
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
