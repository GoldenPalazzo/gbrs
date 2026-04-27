use libretro_backend::{
    AudioVideoInfo, CoreInfo, GameData, JoypadButton, LoadGameResult, PixelFormat, Region,
    RuntimeHandle, libretro_core,
};

use gbrs_engine::{cpu::cpu::Cpu, memory::bus::MemoryBus};

const PALETTE: [u32; 4] = [0xFF_FFFFFF, 0xFF_AAAAAA, 0xFF_555555, 0xFF_000000];
const SAMPLERATE_MCYCLES: f32 = 22.;
const SAMPLES_PER_FRAME: usize = 797;
const SAMPLERATE_HZ: f32 = 1048576. / SAMPLERATE_MCYCLES;
const FPS: f64 = SAMPLERATE_HZ as f64 / SAMPLES_PER_FRAME as f64;

#[inline]
fn as_bytes<T: Copy>(array: &[T]) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(
            core::mem::transmute(array.as_ptr()),
            core::mem::size_of::<T>() * array.len(),
        )
    }
}

#[inline]
fn unit_to_i16(sample: f32) -> i16 {
    assert!((0f32..=1f32).contains(&sample));
    (sample * i16::MAX as f32) as i16
}

struct Emu {
    cpu: Cpu,
    mem: Option<MemoryBus>,
    game_data: Option<GameData>,
    audio_samples: Vec<i16>,
    samples_overflow: f32,
}

impl Emu {
    fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            mem: None,
            game_data: None,
            audio_samples: Vec::new(),
            samples_overflow: 0.,
        }
    }
}

impl Default for Emu {
    fn default() -> Self {
        Emu::new()
    }
}

impl libretro_backend::Core for Emu {
    fn info() -> CoreInfo {
        CoreInfo::new("gAUmeboy", env!("CARGO_PKG_VERSION")).supports_roms_with_extension("gb")
    }

    fn on_load_game(&mut self, game_data: GameData) -> LoadGameResult {
        if game_data.is_empty() {
            return LoadGameResult::Failed(game_data);
        }
        if let Some(path) = game_data.path() {
            let res = MemoryBus::from_file(std::path::Path::new(path));
            match res {
                Err(_) => return LoadGameResult::Failed(game_data),
                Ok(mut mem) => {
                    mem.apu.set_sample_rate(SAMPLERATE_HZ);
                    self.mem = Some(mem);
                }
            };
        } else {
            return LoadGameResult::Failed(game_data);
        }
        let av_info = AudioVideoInfo::new()
            .video(160, 144, FPS, PixelFormat::ARGB8888)
            .audio(SAMPLERATE_HZ as f64)
            .region(Region::NTSC);
        self.game_data = Some(game_data);
        println!("FPS: {FPS}");
        LoadGameResult::Success(av_info)
    }

    fn on_unload_game(&mut self) -> GameData {
        self.game_data.take().unwrap()
    }

    fn on_run(&mut self, handle: &mut RuntimeHandle) {
        if let Some(mem) = &mut self.mem {
            loop {
                let cycles = self.cpu.step(mem);
                mem.step(cycles);
                self.audio_samples.extend(
                    mem.apu
                        .drain_samples()
                        .iter()
                        .map(|sample: &f32| unit_to_i16(*sample)),
                );
                if mem.ppu.frame_ready {
                    mem.ppu.frame_ready = false;
                    let argb: Vec<u32> = mem
                        .ppu
                        .framebuffer
                        .iter()
                        .map(|&p| PALETTE[p as usize])
                        .collect();
                    let fb = as_bytes(&argb);
                    handle.upload_video_frame(fb);
                    self.samples_overflow += SAMPLES_PER_FRAME as f32;
                    let pairs = self.samples_overflow as usize;
                    self.samples_overflow -= pairs as f32;
                    let samples: Vec<i16> = self.audio_samples.drain(..pairs * 2).collect();
                    handle.upload_audio_frame(&samples);

                    break;
                }
            }
        } else {
            unreachable!()
        }
    }

    fn on_reset(&mut self) {}
}

libretro_core!(Emu);
