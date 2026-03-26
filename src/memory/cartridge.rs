pub trait Mapper {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

#[derive(Default)]
pub struct RomOnly {
    data: Vec<u8>,
}

impl Mapper for RomOnly {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3fff => self.data[addr as usize],
            _ => panic!("Invalid read at 0x{:04X}", addr),
        }
    }
    fn write(&mut self, addr: u16, _data: u8) {
        match addr {
            0x0000..=0x3fff => panic!("Invalid write in ROM 0x{:04X}", addr),
            _ => todo!(),
        }
    }
}

#[derive(Default)]
pub struct Mbc1 {
    data: Vec<u8>,
}

impl Mapper for Mbc1 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => self.data[addr as usize],
            _ => panic!("Invalid read at 0x{:04X}", addr),
        }
    }
    fn write(&mut self, addr: u16, _data: u8) {
        match addr {
            0x0000..=0x7fff => panic!("Invalid write in ROM 0x{:04X}", addr),
            _ => todo!(),
        }
    }
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_file(path: &str) -> std::io::Result<Self> {
        let data = std::fs::read(path)?;
        let hw_type = data[0x147];
        let title = String::from_utf8_lossy(&data[0x134..0x144]).to_string();
        let mapper: Box<dyn Mapper> = match hw_type {
            0x00 => Box::new(RomOnly { data }),
            0x01 => Box::new(Mbc1 { data }),
            _ => todo!(),
        };
        Ok(Self {
            title,
            mapper,
        })
    }
}
