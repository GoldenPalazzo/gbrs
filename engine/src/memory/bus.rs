use super::cartridge::Cartridge;
use super::interrupts::{Interrupt, InterruptController};
use super::joypad::Joypad;
use super::serial::Serial;
use super::timer::Timer;
use crate::apu::Apu;
use crate::ppu::ppu::Ppu;

pub struct MemoryBus {
    pub cart: Cartridge,
    wram: [u8; 0x2000],
    hram: [u8; 127],

    pub serial: Serial,
    pub interrupts: InterruptController,
    timer: Timer,
    pub ppu: Ppu,
    pub joypad: Joypad,
    pub apu: Apu,
}

impl Default for MemoryBus {
    fn default() -> Self {
        Self {
            cart: Cartridge::default(),
            wram: [0u8; 0x2000],
            hram: [0u8; 127],
            serial: Serial::default(),
            timer: Timer::default(),
            interrupts: InterruptController::default(),
            ppu: Ppu::default(),
            joypad: Joypad::default(),
            apu: Apu::new(0),
        }
    }
}

impl MemoryBus {
    #[cfg(feature = "std")]
    pub fn from_file(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            cart: Cartridge::from_file(path)?,
            ..Default::default()
        })
    }

    pub fn from_static(rom: &'static [u8]) -> Self {
        Self {
            cart: Cartridge::from_static(rom),
            ..Default::default()
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => self.cart.mapper.read(addr),
            0x8000..=0x9fff => self.ppu.read(addr),
            0xa000..=0xbfff => self.cart.mapper.read(addr),
            0xc000..=0xdfff => self.wram[(addr as usize) - 0xc000],
            0xe000..=0xfdff => self.wram[(addr as usize) - 0xe000], // echo ram
            // 0xd000..=0xdfff => self.switchable_wram[(addr as usize) - 0xd000],
            0xfe00..=0xfe9f => self.ppu.read(addr),
            0xff00 => self.joypad.read(addr),
            0xff04..=0xff07 => self.timer.read(addr),
            0xff10..=0xff26 => self.apu.read(addr),
            0xff30..=0xff3f => self.apu.read(addr),
            0xff40..=0xff4b => self.ppu.read(addr),
            0xff0f | 0xffff => self.interrupts.read(addr),
            0xff80..=0xfffe => self.hram[(addr as usize) - 0xff80],

            _ => 0xff,
        }
    }

    pub fn write8(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x7fff => self.cart.mapper.write(addr, data),
            0x8000..=0x9fff => self.ppu.write(addr, data),
            0xa000..=0xbfff => self.cart.mapper.write(addr, data),
            0xc000..=0xdfff => self.wram[(addr as usize) - 0xc000] = data,
            0xff00 => self.joypad.write(addr, data),
            0xff01 | 0xff02 => self.serial.write(addr, data),
            0xff04..=0xff07 => self.timer.write(addr, data),
            0xff10..=0xff26 => self.apu.write(addr, data),
            0xff30..=0xff3f => self.apu.write(addr, data),
            0xff0f | 0xffff => self.interrupts.write(addr, data),
            0xe000..=0xfdff => self.wram[(addr as usize) - 0xe000] = data, // echo ram
            0xff46 => {
                // OAM DMA transfer
                let dma_base = (data as u16) << 8;
                for dma_off in 0..0xa0 {
                    let dma_data = self.read(dma_base + dma_off);
                    self.ppu.write(0xfe00 + dma_off, dma_data);
                }
            }
            0xfe00..=0xfe9f => self.ppu.write(addr, data),
            0xfea0..=0xfeff => {} // prohibited memory
            0xff40..=0xff4b => self.ppu.write(addr, data),
            0xff80..=0xfffe => self.hram[(addr as usize) - 0xff80] = data,
            _ => {}
        }
    }

    pub fn write16(&mut self, addr: u16, data: u16) {
        let bytes = data.to_le_bytes();
        self.write8(addr, bytes[0]);
        self.write8(addr.wrapping_add(1), bytes[1]);
    }

    pub fn step(&mut self, mcycles: u8) {
        self.serial.step(mcycles);
        self.apu.step(mcycles);
        let ints = self.ppu.step(mcycles);
        if ints & Interrupt::VBlank as u8 != 0 {
            self.interrupts.request(Interrupt::VBlank);
        }
        if ints & Interrupt::LcdStat as u8 != 0 {
            self.interrupts.request(Interrupt::LcdStat);
        }
        let timer = self.timer.step(mcycles);
        if timer.interrupt {
            self.interrupts.request(Interrupt::Timer);
        }
        if timer.apu_tick {
            self.apu.divapu_tick();
        }
    }
}
