use crate::memory::interrupts::Interrupt;

#[derive(Default, Debug, Clone, Copy)]
#[repr(u8)]
pub enum PpuState {
    HorizontalBlank,
    VerticalBlank,
    #[default]
    OAMScan,
    DrawingPixels
}

const DPS: i32 = 456; // dots per scanline
const DPF: i32 = 70224;

// LCDC flags
const POWER_FLAG: u8 = 0x80;
const WIN_TILE_MAP_AREA_FLAG: u8 = 0x40;
const WIN_ENABLE_FLAG: u8 = 0x20;
const BG_WIN_TILE_DATA_AREA_FLAG: u8 = 0x10;
const BG_TILE_MAP_AREA_FLAG: u8 = 0x08;
const OBJ_SIZE_FLAG: u8 = 0x04;
const OBJ_ENABLE_FLAG: u8 = 0x02;
const BG_WIN_ENABLE_PRIO_FLAG: u8 = 0x01;

// STAT flags
const LYC_INT_SEL_FLAG: u8 = 0x40;
const MODE2_INT_SEL_FLAG: u8 = 0x20;
const MODE1_INT_SEL_FLAG: u8 = 0x10;
const MODE0_INT_SEL_FLAG: u8 = 0x08;
const LYC_LC_FLAG: u8 = 0x04; // readonly
const PPU_MODE: u8 = 0x03; // readonly

const STAT_ADDR: u16 = 0xff41;
const SCY_ADDR: u16 = 0xff42;
const SCX_ADDR: u16 = 0xff43;
const LY_ADDR: u16 = 0xff44;
const LYC_ADDR: u16 = 0xff45;
const WY_ADDR: u16 = 0xff4a;
const WX_ADDR: u16 = 0xff4b;

pub struct Ppu {
    pub framebuffer: [u8; 160 * 144],
    pub frame_ready: bool,
    lcdc: u8,

    ly: u8,
    lyc: u8,
    scx: u8,
    scy: u8,
    wy: u8,
    wx: u8,
    stat: u8,

    // Slowdowns not implemented
    state: PpuState,

    dots: u16,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            framebuffer: [0u8; 160 * 144],
            frame_ready: false,
            lcdc: 0,
            ly: 0,
            lyc: 0,
            scx: 0,
            scy: 0,
            wx: 0,
            wy: 0,
            stat: 0,
            state: PpuState::default(),
            dots: 0,
        }
    }
}

impl Ppu {
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            STAT_ADDR => self.stat,
            SCY_ADDR => self.scy,
            SCX_ADDR => self.scx,
            LY_ADDR => self.ly,
            LYC_ADDR => self.lyc,
            WY_ADDR => self.wy,
            WX_ADDR => self.wx,
            _ => unreachable!(),
        }
    }

    pub fn step(&mut self, mcycles: u8) -> Option<Interrupt> {
        let mut int = None;
        self.dots += mcycles as u16 * 4;
        match self.state {
            PpuState::HorizontalBlank => {
                if self.dots >= 204 {
                    self.dots -= 204;
                    self.ly += 1;
                    if self.ly == self.lyc {
                        int = Some(Interrupt::LcdStat);
                    }
                    self.state = if self.ly == 144 {
                        PpuState::VerticalBlank
                    } else {
                        PpuState::OAMScan
                    };
                }
            },
            PpuState::VerticalBlank => {},
            PpuState::OAMScan => {
                if self.dots >= 80 {
                    self.dots -= 80;
                    self.state = PpuState::DrawingPixels;
                }
            },
            PpuState::DrawingPixels => {
                if self.dots >= 172 {
                    self.dots -= 172;
                    self.state = PpuState::HorizontalBlank;
                }
            },
        };

        int
    }

    pub fn get_state(&self) -> PpuState {
        self.state
    }
}
