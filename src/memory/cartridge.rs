use crate::memory::{romonly::RomOnly, mbc1::Mbc1};

pub trait Mapper {
    fn set_rom(&mut self, rom: Vec<u8>);
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

pub struct Cartridge {
    pub title: String,
    pub mapper: Box<dyn Mapper>,
}

impl Default for Cartridge {
    fn default() -> Self {
        Self {
            title: String::default(),
            mapper: Box::new(RomOnly::default()),
        }
    }
}

impl Cartridge {
    pub fn from_file(path: &str) -> std::io::Result<Self> {
        let data = std::fs::read(path)?;
        let hw_type = data[0x147];
        let title = String::from_utf8_lossy(&data[0x134..0x144]).to_string();
        let mut mapper: Box<dyn Mapper> = match hw_type {
            0x00 => Box::new(RomOnly::default()),
            0x01 => Box::new(Mbc1::new(false, false)),
            0x02 => Box::new(Mbc1::new(true, false)),
            0x03 => Box::new(Mbc1::new(true, true)),
            _ => todo!("Mapper {} not implemented", hw_type),
        };
        mapper.set_rom(data);
        println!("Mapper {} rom", hw_type);
        Ok(Self { title, mapper })
    }
}
