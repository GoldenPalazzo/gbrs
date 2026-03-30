pub trait Mapper {
    fn set_rom(&mut self, rom: Vec<u8>);
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

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

pub struct Mbc1 {
    rom: Vec<u8>,
    ram: [u8; 0x6000],

    ram_enable: bool,
    rom_bank: u8,
    ram_bank_rom_upper: u8,
    is_advanced_banking_mode: bool,

    has_ram: bool,
    has_battery: bool,
}

impl Default for Mbc1 {
    fn default() -> Self {
        Self {
            rom: Vec::new(),
            ram: [0u8; 0x6000],
            ram_enable: false,
            rom_bank: 1,
            ram_bank_rom_upper: 0,
            is_advanced_banking_mode: false,
            has_ram: false,
            has_battery: false,
        }
    }
}

impl Mapper for Mbc1 {
    fn set_rom(&mut self, rom: Vec<u8>) {
        self.rom = rom;
    }
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3fff => self.rom[addr as usize],
            0x4000..=0x7fff => {
                let used_bank =
                    self.rom_bank.max(1) as usize | (self.ram_bank_rom_upper as usize) << 5;
                self.rom[addr as usize - 0x4000 + used_bank * 0x4000]
            }
            0xa000..=0xbfff => {
                if self.has_ram && self.ram_enable {
                    let used_bank = if self.is_advanced_banking_mode {
                        0
                    } else {
                        self.ram_bank_rom_upper as usize
                    };
                    self.ram[addr as usize - 0xa000 + used_bank * 0x2000]
                } else {
                    0xff
                }
            }
            _ => panic!("Invalid read at 0x{:04X}", addr),
        }
    }
    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1fff => self.ram_enable = data & 0xf == 0xa,
            0x2000..=0x3fff => self.rom_bank = data & 0x1f,
            0x4000..=0x5fff => self.ram_bank_rom_upper = data & 0x3,
            0x6000..=0x7fff => self.is_advanced_banking_mode = (data & 1) == 1,
            0xa000..=0xbfff => {
                if self.has_ram && self.ram_enable {
                    let used_bank = if self.is_advanced_banking_mode {
                        0
                    } else {
                        self.ram_bank_rom_upper as usize
                    };
                    self.ram[addr as usize - 0xa000 + used_bank * 0x2000] = data;
                }
            }

            // 0x0000..=0x7fff => panic!("Invalid write in ROM 0x{:04X}", addr),
            _ => todo!("Write 0x{:04X}", addr),
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
        let mut mapper: Box<dyn Mapper> = match hw_type {
            0x00 => Box::new(RomOnly::default()),
            0x01 => Box::new(Mbc1::default()),
            0x02 => Box::new(Mbc1 {
                has_ram: true,
                has_battery: false,
                ..Default::default()
            }),
            0x03 => Box::new(Mbc1 {
                has_ram: true,
                has_battery: true,
                ..Default::default()
            }),
            _ => todo!("Mapper {} not implemented", hw_type),
        };
        mapper.set_rom(data);
        println!("Mapper {} rom", hw_type);
        Ok(Self { title, mapper })
    }
}
