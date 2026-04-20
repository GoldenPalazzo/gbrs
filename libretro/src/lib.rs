use libretro_backend::{
    AudioVideoInfo, CoreInfo, GameData, JoypadButton, LoadGameResult, PixelFormat, Region,
    RuntimeHandle, libretro_core,
};

use gbrs_engine::{cpu::cpu::Cpu, memory::bus::MemoryBus};

const PALETTE: [u32; 4] = [0xFF_FFFFFF, 0xFF_AAAAAA, 0xFF_555555, 0xFF_000000];
const FPS: f64 = 59.7;
const SAMPLERATE: f32 = 1048576. / 23.;
const NEEDED_PAIRS: f32 = SAMPLERATE / FPS as f32;

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
    let bi = sample * 2. - 1.;
    (bi * i16::MAX as f32) as i16
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
                    mem.apu.set_sample_rate(SAMPLERATE);
                    self.mem = Some(mem);
                }
            };
        } else {
            return LoadGameResult::Failed(game_data);
        }
        let av_info = AudioVideoInfo::new()
            .video(160, 144, FPS, PixelFormat::ARGB8888)
            .audio(SAMPLERATE as f64)
            .region(Region::NTSC);
        self.game_data = Some(game_data);
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
                // if self.audio_samples.len() as f32 >= NEEDED_SAMPLES {
                //     let s = core::mem::take(&mut self.audio_samples);
                //     println!("Sending {} samples...", s.len());
                //     handle.upload_audio_frame(&s);
                // }
                if mem.ppu.frame_ready {
                    mem.ppu.frame_ready = false;
                    let argb: Vec<u32> = mem
                        .ppu
                        .framebuffer
                        .iter()
                        .map(|&p| PALETTE[p as usize])
                        .collect();
                    // let samples: Vec<i16> = mem
                    //     .apu
                    //     .drain_samples()
                    //     .iter()
                    //     .map(|&s| unit_to_i16(s))
                    //     .collect();
                    // println!("Sending {} samples", samples.len());
                    let fb = as_bytes(&argb);
                    handle.upload_video_frame(fb);
                    self.samples_overflow += NEEDED_PAIRS.fract();
                    if self.samples_overflow >= 1. {
                        self.samples_overflow -= 1.;
                    }
                    let pairs = (NEEDED_PAIRS + self.samples_overflow) as usize;
                    let n = (pairs * 2).min(self.audio_samples.len());
                    println!(
                        "buffer: {}, n: {}, overflow: {}",
                        self.audio_samples.len(),
                        n,
                        self.samples_overflow
                    );
                    let samples: Vec<i16> = self.audio_samples.drain(..n).collect();
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
