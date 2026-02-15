use crate::memory::cartridge::Cartridge;

// [derive(Default)]
pub struct MemoryBus {
    pub cart: Cartridge,
    pub wram: [u8; 0x1000]
}

impl Default for MemoryBus {
    fn default() -> Self {
        Self {
            cart: Cartridge::default(),
            wram: [0u8; 0x1000]
        }
    }
}

impl MemoryBus {
    // pub fn new() -> Self {
    //     Self::default()
    // }

    pub fn from_file(path: &str) -> Self {
        Self {
            cart: Cartridge::from_file(path).unwrap(),
            wram: [0u8; 0x1000]
        }
    }


    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7fff => self.cart.mapper.read(addr),
            0xc000..=0xcfff => self.wram[(addr as usize)-0xc000],
            _ => 0xff
        }
    }

    pub fn write8(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x7fff => self.cart.mapper.write(addr, data),
            0xc000..=0xcfff => self.wram[(addr as usize)-0xc000] = data,
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
}
