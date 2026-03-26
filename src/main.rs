mod cpu;
mod memory;
use crate::cpu::cpu::Cpu;
use crate::memory::memory::MemoryBus;

use std::env;
use std::io::{self, Write};

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

    let mut cpu = Cpu::new();
    let mut mem = MemoryBus::from_file(&args[1]).unwrap();
    println!("Loaded cart {:?}", mem.cart.title);
    let mut bp = false;
    // let bps = vec![0x20f];
    let bps = [];

    loop {
        if bps.contains(&cpu.regs.get_pc()) {
            bp = true;
        }
        if bp {
            wait_for_enter();
        }
        let cycles = cpu.step(&mut mem);
        mem.step(cycles);
    }
}
