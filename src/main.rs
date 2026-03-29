mod cpu;
mod memory;
mod ppu;
use crate::cpu::cpu::Cpu;
use crate::memory::memory::MemoryBus;

use std::env;
use std::io::{self, Write};

use minifb::{Window, WindowOptions, Scale};

fn wait_for_enter() {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
}

fn main() -> std::io::Result<()> {
    // env_logger::init();

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
    // debug!("args: {:#?}", args);
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
        }
    ).unwrap();
    window.update();

    let mut cpu = Cpu::new();
    let mut mem = MemoryBus::from_file(&args[1]).unwrap();
    println!("Loaded cart {:?}", mem.cart.title);
    let mut bp = false;
    // let bps = vec![0x20f];
    let bps = [];
    const PALETTE: [u32; 4] = [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000];
    loop {
        
        if bps.contains(&cpu.regs.get_pc()) {
            bp = true;
        }
        if bp {
            wait_for_enter();
        }
        let cycles = cpu.step(&mut mem);
        mem.step(cycles);
        if mem.ppu.frame_ready {
            mem.ppu.frame_ready = false;
            let rgb: Vec<u32> = mem.ppu.framebuffer.iter()
                .map(|&p| PALETTE[p as usize])
                .collect();
            window.update_with_buffer(&rgb, 160, 144).unwrap();
            // println!("Presenting frame!");
        }
    }
}
