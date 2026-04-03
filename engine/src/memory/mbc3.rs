use super::cartridge::Mapper;
use super::rtc::{RtcSource, MockRtc};
use alloc::{boxed::Box, vec::Vec};
// use chrono::{DateTime, Local, Timelike, Datelike};

pub struct Mbc3 {
    rom: Vec<u8>,
    ram: [u8; 0x8000],

    ram_timer_enable: bool,
    rom_bank: u8,
    ram_rtc_select: u8,
    latch_rtc: u8,
    num_banks: usize,

    // latest_rtc_snapshot: DateTime<Local>,
    latest_rtc_snapshot: Box<dyn RtcSource>,
    has_ram: bool,
    has_timer: bool,
    has_battery: bool,
}

impl Mbc3 {
    pub fn new(has_ram: bool, has_timer: bool, has_battery: bool) -> Self {
        Self {
            has_ram,
            has_timer,
            has_battery,
            ..Default::default()
        }
    }
}

impl Default for Mbc3 {
    fn default() -> Self {
        Self {
            rom: Vec::new(),
            ram: [0u8; 0x8000],
            ram_timer_enable: false,
            rom_bank: 1,
            ram_rtc_select: 0,
            latch_rtc: 67, // enforce correct latching procedure, 00 -> 01
            latest_rtc_snapshot: Box::new(MockRtc{}),
            num_banks: 1,
            has_ram: false,
            has_timer: false,
            has_battery: false,
        }
    }
}

impl Mapper for Mbc3 {
    fn set_rom(&mut self, rom: Vec<u8>) {
        self.rom = rom;
        self.num_banks = (self.rom.len() / 0x4000).max(1) as usize;
    }
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3fff => self.rom[addr as usize],
            0x4000..=0x7fff => {
                let bank = self.rom_bank.max(1) as usize;
                let used_bank = bank % self.num_banks;
                self.rom[addr as usize - 0x4000 + used_bank * 0x4000]
            },
            0xa000..=0xbfff => match (self.has_ram, self.has_timer, self.ram_rtc_select) {
                (true, _, 0x00..=0x07) => self.ram[addr as usize - 0xa000 + self.ram_rtc_select as usize * 0x2000],
                (_, true, 0x08) => self.latest_rtc_snapshot.second() as u8,
                (_, true, 0x09) => self.latest_rtc_snapshot.minute() as u8,
                (_, true, 0x0a) => self.latest_rtc_snapshot.hour() as u8,
                (_, true, 0x0b) => (self.latest_rtc_snapshot.ordinal0() & 0xff) as u8,
                // TODO: need to handle carry and halt
                (_, true, 0x0c) => (self.latest_rtc_snapshot.ordinal0() >> 8 & 1) as u8,
                _ => 0xff
            }
            _ => 0xff,
        }
    }
    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1fff => self.ram_timer_enable = data == 0xa,
            0x2000..=0x3fff => self.rom_bank = data & 0x7f,
            0x4000..=0x5fff => self.ram_rtc_select = data,
            0x6000..=0x7fff => {
                if !self.has_timer {return;}
                if self.latch_rtc == 0 && data == 1 {
                    self.latest_rtc_snapshot.snapshot();
                }
                self.latch_rtc = data;
            },
            0xa000..=0xbfff => match (self.ram_timer_enable, self.ram_rtc_select) {
                (true, 0x00..=0x07) => self.ram[addr as usize - 0xa000 + self.ram_rtc_select as usize * 0x2000] = data,
                // (true, 0x08..=0x0c) => ,
                _ => {}
            }
            _ => unreachable!("Invalid write at 0x{:04X}", addr),
        }
    }
}


