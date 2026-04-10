use alloc::{boxed::Box, vec::Vec};

pub struct Mbc1 {
    rom: &'static [u8],
    pub ram: [u8; 0x8000],

    ram_enable: bool,
    rom_bank: u8,
    ram_bank_rom_upper: u8,
    is_advanced_banking_mode: bool,
    num_banks: usize,

    has_ram: bool,
    pub has_battery: bool,
    pub dirty_ram: bool,
}

impl Mbc1 {
    pub fn new(has_ram: bool, has_battery: bool) -> Self {
        Self {
            has_ram,
            has_battery,
            ..Default::default()
        }
    }
}

impl Default for Mbc1 {
    fn default() -> Self {
        Self {
            rom: &[],
            ram: [0u8; 0x8000],
            ram_enable: false,
            rom_bank: 1,
            ram_bank_rom_upper: 0,
            is_advanced_banking_mode: false,
            num_banks: 1,
            has_ram: false,
            has_battery: false,
            dirty_ram: false,
        }
    }
}

impl Mbc1 {
    pub fn set_rom(&mut self, rom: Vec<u8>) {
        self.num_banks = (rom.len() / 0x4000).max(1);
        self.rom = Box::leak(rom.into_boxed_slice());
    }
    pub fn set_rom_static(&mut self, rom: &'static [u8]) {
        self.num_banks = (rom.len() / 0x4000).max(1);
        self.rom = rom;
    }
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3fff => {
                let used_bank = if self.is_advanced_banking_mode {
                    ((self.ram_bank_rom_upper as usize) << 5) % self.num_banks
                } else {
                    0
                };
                self.rom[addr as usize + used_bank * 0x4000]
            }
            0x4000..=0x7fff => {
                let used_bank = (self.rom_bank.max(1) as usize
                    | (self.ram_bank_rom_upper as usize) << 5)
                    % self.num_banks;
                self.rom[addr as usize - 0x4000 + used_bank * 0x4000]
            }
            0xa000..=0xbfff => {
                if self.has_ram && self.ram_enable {
                    let used_bank = if self.is_advanced_banking_mode {
                        self.ram_bank_rom_upper as usize
                    } else {
                        0
                    };
                    self.ram[addr as usize - 0xa000 + used_bank * 0x2000]
                } else {
                    0xff
                }
            }
            _ => 0xff,
        }
    }
    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1fff => self.ram_enable = data & 0xf == 0xa,
            0x2000..=0x3fff => self.rom_bank = data & 0x1f,
            0x4000..=0x5fff => self.ram_bank_rom_upper = data & 0x3,
            0x6000..=0x7fff => self.is_advanced_banking_mode = (data & 1) == 1,
            0xa000..=0xbfff => {
                if self.has_ram && self.ram_enable {
                    let used_bank = if self.is_advanced_banking_mode {
                        self.ram_bank_rom_upper as usize
                    } else {
                        0
                    };
                    self.ram[addr as usize - 0xa000 + used_bank * 0x2000] = data;
                    self.dirty_ram = true;
                }
            }
            _ => {}
        }
    }
}
