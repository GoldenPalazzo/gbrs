use crate::memory::interrupts::Interrupt;

#[derive(Default, Debug, Clone, Copy)]
#[repr(u8)]
pub enum PpuState {
    HorizontalBlank,
    VerticalBlank,
    #[default]
    OAMScan,
    DrawingPixels,
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

const LCDC_ADDR: u16 = 0xff40;
const STAT_ADDR: u16 = 0xff41;
const SCY_ADDR: u16 = 0xff42;
const SCX_ADDR: u16 = 0xff43;
const LY_ADDR: u16 = 0xff44;
const LYC_ADDR: u16 = 0xff45;
const BGP_ADDR: u16 = 0xff47;
const OBP0_ADDR: u16 = 0xff48;
const OBP1_ADDR: u16 = 0xff49;
const WY_ADDR: u16 = 0xff4a;
const WX_ADDR: u16 = 0xff4b;
const VRAM_ADDR_START: u16 = 0x8000;
const VRAM_ADDR_END: u16 = 0x9fff;
const OAM_ADDR_START: u16 = 0xfe00;
const OAM_ADDR_END: u16 = 0xfe9f;

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

    bgp: u8,
    obp0: u8,
    obp1: u8,

    /*
     * tmap 1: $9800-$9BFF
     * tmap 2: $9C00-$9FFF
     */
    vram: [u8; 0x2000],
    oam: [u8; 0xa0],

    // Slowdowns not implemented
    state: PpuState,

    dots: u16,
    window_line_cnt: u8,
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
            bgp: 0,
            obp0: 0,
            obp1: 0,
            vram: [0u8; 0x2000],
            oam: [0u8; 0xa0],
            state: PpuState::default(),
            dots: 0,
            window_line_cnt: 0,
        }
    }
}

impl Ppu {
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            LCDC_ADDR => self.lcdc,
            STAT_ADDR => self.stat,
            SCY_ADDR => self.scy,
            SCX_ADDR => self.scx,
            LY_ADDR => self.ly,
            LYC_ADDR => self.lyc,
            WY_ADDR => self.wy,
            WX_ADDR => self.wx,
            BGP_ADDR => self.bgp,
            VRAM_ADDR_START..=VRAM_ADDR_END => self.vram[(addr - VRAM_ADDR_START) as usize],
            OAM_ADDR_START..=OAM_ADDR_END => self.oam[(addr - OAM_ADDR_START) as usize],
            _ => unreachable!("Read at 0x{:04X}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            LCDC_ADDR => self.lcdc = data,
            STAT_ADDR => self.stat = data,
            SCY_ADDR => self.scy = data,
            SCX_ADDR => self.scx = data,
            LY_ADDR => self.ly = data,
            LYC_ADDR => self.lyc = data,
            WY_ADDR => self.wy = data,
            WX_ADDR => self.wx = data,
            BGP_ADDR => self.bgp = data,
            OBP0_ADDR => self.obp0 = data,
            OBP1_ADDR => self.obp1 = data,
            // 0xff48 | 0xff49 => {println!("Stub: write to obj palette {} (data=0x{:02X})", addr - 0xff48, data)}
            VRAM_ADDR_START..=VRAM_ADDR_END => self.vram[(addr - VRAM_ADDR_START) as usize] = data,
            OAM_ADDR_START..=OAM_ADDR_END => self.oam[(addr - OAM_ADDR_START) as usize] = data,
            _ => unreachable!("Write at 0x{:04X} (data=0x{:02X})", addr, data),
        }
    }

    pub fn step(&mut self, mcycles: u8) -> u8 {
        // assert_eq!(self.lcdc & OBJ_SIZE_FLAG, 0);
        let mut int: u8 = 0;
        self.dots += mcycles as u16 * 4;
        match self.state {
            PpuState::HorizontalBlank => {
                if self.dots >= 204 {
                    self.dots -= 204;
                    self.ly += 1;
                    if self.ly == self.lyc {
                        self.stat |= LYC_LC_FLAG;
                        if self.stat & LYC_INT_SEL_FLAG != 0 {
                            int |= Interrupt::LcdStat as u8;
                        }
                    } else {
                        self.stat &= !LYC_LC_FLAG;
                    }
                    if self.ly == 144 {
                        self.state = PpuState::VerticalBlank;
                        self.frame_ready = true;
                        int |= Interrupt::VBlank as u8;
                        if self.stat & MODE1_INT_SEL_FLAG != 0 {
                            int |= Interrupt::LcdStat as u8;
                        }
                    } else {
                        self.state = PpuState::OAMScan;
                        if self.stat & MODE2_INT_SEL_FLAG != 0 {
                            int |= Interrupt::LcdStat as u8;
                        }
                    }
                }
            }
            PpuState::VerticalBlank => {
                if self.dots >= 456 {
                    self.dots -= 456;
                    self.ly += 1;
                    if self.ly > 153 {
                        self.ly = 0;
                        self.window_line_cnt = 0;
                        self.state = PpuState::OAMScan;
                        if self.stat & MODE1_INT_SEL_FLAG != 0 {
                            int |= Interrupt::LcdStat as u8;
                        }
                    }
                }
            }
            PpuState::OAMScan => {
                if self.dots >= 80 {
                    self.dots -= 80;
                    self.state = PpuState::DrawingPixels;
                }
            }
            PpuState::DrawingPixels => {
                if self.dots >= 172 {
                    self.dots -= 172;
                    self.state = PpuState::HorizontalBlank;
                    if self.stat & MODE0_INT_SEL_FLAG != 0 {
                        int |= Interrupt::LcdStat as u8;
                    }
                    self.render_scanline();
                }
            }
        };
        self.stat = (self.stat & !PPU_MODE) | (self.state as u8 & PPU_MODE);
        int
    }

    fn render_scanline(&mut self) {
        if self.lcdc & POWER_FLAG == 0 {
            return;
        }
        let mut drew_win = 0;
        let tile_data_base = match self.lcdc & BG_WIN_TILE_DATA_AREA_FLAG != 0 {
            true => 0x8000 - VRAM_ADDR_START,
            false => 0x9000 - VRAM_ADDR_START,
        } as usize;

        let mut sprites_on_line: Vec<usize> = Vec::new();
        let obj_data_base = 0x8000 - VRAM_ADDR_START as usize;
        let obj_height = if self.lcdc & OBJ_SIZE_FLAG != 0 {
            16u8
        } else {
            8u8
        };
        if self.lcdc & OBJ_ENABLE_FLAG != 0 {
            for spr in 0..40 {
                let y_16 = self.oam[spr * 4] as i32;
                if self.ly as i32 >= y_16 - 16 && (self.ly as i32) < y_16 - 16 + obj_height as i32 {
                    sprites_on_line.push(spr);
                    if sprites_on_line.len() == 10 {
                        break;
                    }
                }
            }
        }            
        for x in 0..160usize {
            if self.lcdc & WIN_ENABLE_FLAG != 0
                && self.lcdc & BG_WIN_ENABLE_PRIO_FLAG != 0
                && self.wx < 167
                && self.wy < 144
                && self.ly >= self.wy
                && x + 7 >= self.wx as usize
            {
                let win_map_base = match self.lcdc & WIN_TILE_MAP_AREA_FLAG != 0 {
                    true => 0x9c00 - VRAM_ADDR_START,
                    false => 0x9800 - VRAM_ADDR_START,
                } as usize;
                let scrolled_x = (x + 7 - self.wx as usize) & 0xff;
                let scrolled_y = self.window_line_cnt as usize;
                let cur_tile_x = scrolled_x / 8;
                let cur_tile_x_pixel = scrolled_x % 8;
                let cur_tile_y = scrolled_y / 8;
                let cur_tile_y_pixel = scrolled_y % 8;

                let tile_index = self.vram[win_map_base + cur_tile_y * 32 + cur_tile_x] as usize;
                let tile_data_ptr = if self.lcdc & BG_WIN_TILE_DATA_AREA_FLAG != 0 {
                    tile_data_base + tile_index * 16
                } else {
                    let signed_index = tile_index as i8 as i32;
                    (tile_data_base as i32 + signed_index * 16) as usize
                };
                let row = [
                    self.vram[tile_data_ptr + cur_tile_y_pixel * 2],
                    self.vram[tile_data_ptr + cur_tile_y_pixel * 2 + 1],
                ];
                let lo = (row[0] >> (7 - cur_tile_x_pixel)) & 1;
                let hi = (row[1] >> (7 - cur_tile_x_pixel)) & 1;
                let color = (hi << 1) | lo;

                assert!((0..4).contains(&color));
                self.framebuffer[self.ly as usize * 160 + x] = (self.bgp >> (2 * color)) & 0b11;
                drew_win = 1;
            } else {
                let bg_map_base = match self.lcdc & BG_TILE_MAP_AREA_FLAG != 0 {
                    true => 0x9c00 - VRAM_ADDR_START,
                    false => 0x9800 - VRAM_ADDR_START,
                } as usize;
                let scrolled_x = (x + self.scx as usize) & 0xff;
                let scrolled_y = (self.ly as usize + self.scy as usize) & 0xff;
                let cur_tile_x = scrolled_x / 8;
                let cur_tile_x_pixel = scrolled_x % 8;
                let cur_tile_y = scrolled_y / 8;
                let cur_tile_y_pixel = scrolled_y % 8;

                let tile_index = self.vram[bg_map_base + cur_tile_y * 32 + cur_tile_x] as usize;
                let tile_data_ptr = if self.lcdc & BG_WIN_TILE_DATA_AREA_FLAG != 0 {
                    tile_data_base + tile_index * 16
                } else {
                    let signed_index = tile_index as i8 as i32;
                    (tile_data_base as i32 + signed_index * 16) as usize
                };
                let row = [
                    self.vram[tile_data_ptr + cur_tile_y_pixel * 2],
                    self.vram[tile_data_ptr + cur_tile_y_pixel * 2 + 1],
                ];
                let lo = (row[0] >> (7 - cur_tile_x_pixel)) & 1;
                let hi = (row[1] >> (7 - cur_tile_x_pixel)) & 1;
                let color = (hi << 1) | lo;

                assert!((0..4).contains(&color));
                self.framebuffer[self.ly as usize * 160 + x] = (self.bgp >> (2 * color)) & 0b11;
            }

            for &spr in &sprites_on_line {

                let y_16 = self.oam[spr * 4];
                if y_16 == 0 || y_16 >= 160 {
                    continue;
                }
                let x_8 = self.oam[spr * 4 + 1];
                if x_8 == 0 || x_8 >= 168 {
                    continue;
                }
                let index = self.oam[spr * 4 + 2];
                let attrs = self.oam[spr * 4 + 3];
                if attrs & 0x80 != 0 {
                    let bg_color = self.framebuffer[self.ly as usize * 160 + x];
                    if bg_color != 0 {
                        continue;
                    }
                }
                if x >= x_8 as usize - 8 && x < x_8 as usize {
                    let mut cur_tile_x_pixel = x - (x_8 as usize - 8);
                    let mut cur_tile_y_pixel = self.ly as usize - (y_16 as usize - 16);
                    if attrs & 0x40 != 0 {
                        cur_tile_y_pixel = obj_height as usize - 1 - cur_tile_y_pixel;
                    }
                    if attrs & 0x20 != 0 {
                        cur_tile_x_pixel = 7 - cur_tile_x_pixel;
                    }

                    let tile_index = if obj_height == 16 {
                        index & 0xFE
                    } else {
                        index
                    } as usize;
                    let tile_data_ptr = obj_data_base + tile_index * 16;
                    let row = [
                        self.vram[tile_data_ptr + cur_tile_y_pixel * 2],
                        self.vram[tile_data_ptr + cur_tile_y_pixel * 2 + 1],
                    ];
                    let lo = (row[0] >> (7 - cur_tile_x_pixel)) & 1;
                    let hi = (row[1] >> (7 - cur_tile_x_pixel)) & 1;
                    let color = (hi << 1) | lo;
                    if color == 0 {
                        continue;
                    }
                    let palette = match attrs & 0x10 != 0 {
                        true => self.obp1,
                        false => self.obp0,
                    };

                    assert!((0..4).contains(&color));
                    self.framebuffer[self.ly as usize * 160 + x] = (palette >> (2 * color)) & 0b11;
                    break;
                }
            }
        }
        self.window_line_cnt += drew_win;
    }
}
