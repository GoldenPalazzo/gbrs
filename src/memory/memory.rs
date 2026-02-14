#[derive(Default)]
pub struct Header {
    pub title: String,
    // pub manufacturer: String,
    // pub cgb_only: bool
}

pub struct Cartridge {
    pub header: Header,
    pub data: [u8; 0x4000]
}

impl Default for Cartridge {
    fn default() -> Self {
        Self {
            data: [0u8; 0x4000],
            header: Header::default()
        }
    }
}

impl Cartridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&mut self, path: &str) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Read;
        let mut file = File::open(path)?;
        file.read_exact(&mut self.data)?;
        self.load_header();
        Ok(())
    }

    pub fn from_file(path: &str) -> std::io::Result<Self> {
        let mut new = Self::new();
        new.load(path)?;
        Ok(new)
    }

    fn load_header(&mut self) {
        self.header.title = 
            String::from_utf8_lossy(&self.data[0x134..0x144])
            .to_string();
    }
}

#[derive(Default)]
pub struct MemoryBus {
    pub cart: Cartridge
}

impl MemoryBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3fff => self.cart.data[addr as usize],
            _ => 0xff
        }
    }
}
