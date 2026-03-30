use crate::memory::cartridge::Cartridge;
use crate::memory::interrupts::{Interrupt, InterruptController};
use crate::memory::serial::Serial;
use crate::memory::timer::Timer;
use crate::ppu::ppu::Ppu;

// [derive(Default)]
pub struct MemoryBus {
    pub cart: Cartridge,
    wram: [u8; 0x1000],
    switchable_wram: [u8; 0x1000],
    hram: [u8; 127],

    pub serial: Serial,
    pub interrupts: InterruptController,
    timer: Timer,
    pub ppu: Ppu,
}

impl Default for MemoryBus {
    fn default() -> Self {
        Self {
            cart: Cartridge::default(),
            wram: [0u8; 0x1000],
            switchable_wram: [0u8; 0x1000],
            hram: [0u8; 127],
            serial: Serial::default(),
            timer: Timer::default(),
            interrupts: InterruptController::default(),
            ppu: Ppu::default(),
        }
    }
}

impl MemoryBus {
    // pub fn new() -> Self {
    //     Self::default()
    // }

    pub fn from_file(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            cart: Cartridge::from_file(path)?,
            ..Default::default()
        })
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => self.cart.mapper.read(addr),
            0x8000..=0x9fff => self.ppu.read(addr),
            0xc000..=0xcfff => self.wram[(addr as usize) - 0xc000],
            0xd000..=0xdfff => self.switchable_wram[(addr as usize) - 0xd000],
            0xfe00..=0xfe9f => self.ppu.read(addr),
            0xff00 => {
                println!("Stub: read in 0x{:02X} (gamepad)", addr);
                0xff
            }
            0xff04..=0xff07 => self.timer.read(addr),
            0xff10..=0xff26 => {
                println!("Stub: read in 0x{:02X}", addr);
                0
            }
            0xff40..=0xff4b => self.ppu.read(addr),
            0xff0f | 0xffff => self.interrupts.read(addr),
            0xff80..=0xfffe => self.hram[(addr as usize) - 0xff80],

            // 0xff44 => 0x90, // stubbed to pass cpu_instrs
            _ => 0xff,
        }
    }

    pub fn write8(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x7fff => self.cart.mapper.write(addr, data),
            0x8000..=0x9fff => self.ppu.write(addr, data),
            0xc000..=0xcfff => self.wram[(addr as usize) - 0xc000] = data,
            0xd000..=0xdfff => self.switchable_wram[(addr as usize) - 0xd000] = data,
            0xff00 => {
                println!("Stub: write in 0x{:02X} (gamepad)", addr)
            }
            0xff01 | 0xff02 => self.serial.write(addr, data),
            0xff04..=0xff07 => self.timer.write(addr, data),
            0xff10..=0xff26 => {} //audio
            0xff0f | 0xffff => self.interrupts.write(addr, data),
            0xe000..=0xfdff => panic!("Tried to write in echo ram 0x{:02X}!", addr),
            0xfe00..=0xfe9f => self.ppu.write(addr, data),
            0xfea0..=0xfeff => println!("Write in not usable mem 0x{:04X} ignored", addr),
            0xff40..=0xff4b => self.ppu.write(addr, data),
            0xff80..=0xfffe => self.hram[(addr as usize) - 0xff80] = data,
            _ => todo!(
                "Write (data=0x{:02x}) to address 0x{:04X} hasn't been implemented yet",
                data,
                addr
            ),
        }
    }

    pub fn write16(&mut self, addr: u16, data: u16) {
        let bytes = data.to_le_bytes();
        self.write8(addr, bytes[0]);
        self.write8(addr.wrapping_add(1), bytes[1]);
    }

    pub fn step(&mut self, mcycles: u8) {
        self.serial.step(mcycles);
        let ints = self.ppu.step(mcycles);
        if ints & Interrupt::VBlank as u8 != 0 {
            self.interrupts.request(Interrupt::VBlank);
        }
        if ints & Interrupt::LcdStat as u8 != 0 {
            self.interrupts.request(Interrupt::LcdStat);
        }
        if self.timer.step(mcycles) {
            self.interrupts.request(Interrupt::Timer);
        }
    }
}
