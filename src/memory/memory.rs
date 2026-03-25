use crate::memory::cartridge::Cartridge;
use crate::memory::io::Serial;
use crate::memory::timer::Timer;

// [derive(Default)]
pub struct MemoryBus {
    pub cart: Cartridge,
    wram: [u8; 0x1000],
    switchable_wram: [u8; 0x1000],
    hram: [u8; 127],
    
    pub serial: Serial,
    timer: Timer,
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
            0xc000..=0xcfff => self.wram[(addr as usize)-0xc000],
            0xd000..=0xdfff => self.switchable_wram[(addr as usize)-0xd000],
            0xff04..=0xff07 => self.timer.read(addr),
            0xff80..=0xfffe => self.hram[(addr as usize)-0xff80],
            _ => 0xff
        }
    }

    pub fn write8(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x7fff => self.cart.mapper.write(addr, data),
            0xc000..=0xcfff => self.wram[(addr as usize)-0xc000] = data,
            0xd000..=0xdfff => self.switchable_wram[(addr as usize)-0xd000] = data,
            0xff04..=0xff07 => self.timer.write(addr, data),
            0xe000..=0xfdff => panic!("Tried to write in echo ram 0x{:02X}!", addr),
            0xff80..=0xfffe => self.hram[(addr as usize)-0xff80] = data,
            _ => todo!(
                "Write to address 0x{:04X} hasn't been implemented yet",
                addr
            )
        } 
    }

    pub fn write16(&mut self, addr: u16, data: u16) {
        let bytes = data.to_le_bytes();
        self.write8(addr, bytes[0]);
        self.write8(addr.wrapping_add(1), bytes[1]);
    }

    pub fn step(&mut self, mcycles: u8) {
        self.timer.step(mcycles);
    }
}
