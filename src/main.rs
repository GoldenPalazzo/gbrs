mod cpu;
mod memory;
mod ppu;
use crate::cpu::cpu::Cpu;
use crate::memory::memory::MemoryBus;

use std::env;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use minifb::{Key, Scale, Window, WindowOptions};

fn wait_for_enter() {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
}

fn main_loop(mut cpu: Cpu, mut mem: MemoryBus, mut win: Window, bps: &Option<Vec<u16>>) {
    const PALETTE: [u32; 4] = [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000];
    const FRAME_DURATION: Duration = Duration::from_nanos(16_742_706);
    let mut frame_start = Instant::now();
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
        let cycles = cpu.step(&mut mem);
        mem.step(cycles);
        if mem.ppu.frame_ready {
            mem.ppu.frame_ready = false;
            let rgb: Vec<u32> = mem
                .ppu
                .framebuffer
                .iter()
                .map(|&p| PALETTE[p as usize])
                .collect();
            win.update_with_buffer(&rgb, 160, 144).unwrap();
            // println!("Presenting frame!");
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
            let elapsed = frame_start.elapsed();
            if elapsed < FRAME_DURATION {
                std::thread::sleep(FRAME_DURATION - elapsed);
            }
            frame_start = Instant::now();
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

    let mut cpu = Cpu::new();
    let mut mem = MemoryBus::from_file(&args[1]).unwrap();
    println!("Loaded cart {:?}", mem.cart.title);
    // let bps = vec![0x20f];
    // let bps = [];
    main_loop(cpu, mem, window, &None);
    Ok(())
}
