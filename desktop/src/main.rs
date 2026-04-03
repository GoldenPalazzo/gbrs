use gbrs_engine::cpu::cpu::Cpu;
use gbrs_engine::memory::bus::MemoryBus;

use std::env;
use std::io::{self, Write};
use std::time::Duration;

use minifb::{Key, Scale, Window, WindowOptions};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub type SampleBuffer = Arc<Mutex<Vec<f32>>>;

pub struct AudioOutput {
    _stream: cpal::Stream, // mantieni vivo lo stream
    pub buffer: SampleBuffer,

    sample_rate: f32,
}

impl AudioOutput {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();
        let sample_rate = config.sample_rate() as f32;

        let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let buffer_clone = buffer.clone();

        let stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _| {
                    let mut buf = buffer_clone.lock().unwrap();
                    for sample in data.iter_mut() {
                        *sample = if !buf.is_empty() {
                            buf.remove(0)
                        } else {
                            0.0 // silenzio se il buffer è vuoto
                        };
                    }
                },
                |err| eprintln!("Audio error: {err}"),
                None,
            )
            .unwrap();

        stream.play().unwrap();
        Self {
            _stream: stream,
            buffer,
            sample_rate,
        }
    }

    pub fn get_sample_rate(&self) -> f32 {
        self.sample_rate
    }
}

impl Default for AudioOutput {
    fn default() -> Self {
        Self::new()
    }
}

fn wait_for_enter() {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
}

fn main_loop(
    mut cpu: Cpu,
    mut mem: MemoryBus,
    mut win: Window,
    audio: AudioOutput,
    bps: &Option<Vec<u16>>,
) {
    const PALETTE: [u32; 4] = [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000];
    let mut bp = false;
    loop {
        if let Some(a) = bps
            && a.contains(&cpu.regs.get_pc())
        {
            bp = true;
        }
        if bp {
            wait_for_enter();
        }

        if audio.buffer.lock().unwrap().len() > 4096 {
            std::thread::sleep(Duration::from_millis(1));
        }

        let cycles = cpu.step(&mut mem);
        mem.step(cycles);
        audio.buffer.lock().unwrap().extend(mem.apu.drain_samples());
        if mem.ppu.frame_ready {
            mem.ppu.frame_ready = false;
            let rgb: Vec<u32> = mem
                .ppu
                .framebuffer
                .iter()
                .map(|&p| PALETTE[p as usize])
                .collect();
            win.update_with_buffer(&rgb, 160, 144).unwrap();
            if let Some(int) = mem.joypad.set_buttons(
                win.is_key_down(Key::X),
                win.is_key_down(Key::Z),
                win.is_key_down(Key::Space),
                win.is_key_down(Key::Enter),
            ) {
                mem.interrupts.request(int);
            }
            if let Some(int) = mem.joypad.set_dpad(
                win.is_key_down(Key::Right),
                win.is_key_down(Key::Left),
                win.is_key_down(Key::Up),
                win.is_key_down(Key::Down),
            ) {
                mem.interrupts.request(int);
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let log_file = std::fs::File::create("cpu.log").unwrap();
    env_logger::Builder::new()
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .filter_level(log::LevelFilter::Debug)
        .format(|buf, record| {
            use std::io::Write;
            writeln!(buf, "{}", record.args())
        })
        .init();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("No program provided!");
    }

    let mut window = Window::new(
        "golden boy",
        160,
        144,
        WindowOptions {
            scale: Scale::X8,
            ..WindowOptions::default()
        },
    )
    .unwrap();
    window.update();

    let cpu = Cpu::new();
    let mut mem = MemoryBus::from_file(&args[1]).unwrap();
    let audio = AudioOutput::new();
    mem.apu.set_sample_rate(audio.get_sample_rate());

    main_loop(cpu, mem, window, audio, &None);
    Ok(())
}
