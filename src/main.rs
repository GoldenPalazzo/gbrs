mod cpu;
use crate::cpu::cpu::CPU;

use std::io::{self, Write};

fn wait_for_enter() {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
}

fn main() {
    env_logger::init();

    let buf = [0u8; 1024];
    let mut cpu = CPU::new();

    loop {
        wait_for_enter();
        cpu.step(&buf);
    }
}
