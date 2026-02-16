mod cpu;
mod memory;
use crate::cpu::cpu::CPU;
use crate::memory::memory::MemoryBus;

use log::debug;
use std::io::{self, Write};
use std::env;

fn wait_for_enter() {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
}

fn main() -> std::io::Result<()> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    debug!("args: {:#?}", args);
    if args.len() < 2 {
        panic!("No program provided!");
    }

    let mut cpu = CPU::new();
    let mut mem = MemoryBus::from_file(&args[1]);
    println!("Loaded cart {:?}", mem.cart.title);
    let mut bp = false;
    let bps = vec![0x20f];

    loop {
        if bps.contains(&cpu.regs.get_pc()) { bp = true; }
        if bp {
            wait_for_enter();
        }
        cpu.step(&mut mem);
    }
}
