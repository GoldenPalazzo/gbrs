mod cpu;
mod memory;
use crate::cpu::cpu::CPU;
use crate::memory::memory::MemoryBus;

use std::io::{self, Write};

fn wait_for_enter() {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
}

fn main() -> std::io::Result<()> {
    env_logger::init();

    let mut cpu = CPU::new();
    let mut mem = MemoryBus::new();
    mem.cart.load("./example.gb")?;
    println!("Loaded cart {:?}", mem.cart.header.title);

    loop {
        wait_for_enter();
        cpu.step(&mem);
    }
}
