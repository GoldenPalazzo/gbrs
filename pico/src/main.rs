#![no_std]
#![no_main]
use gbrs_engine::cpu::cpu::Cpu;
use gbrs_engine::memory::bus::MemoryBus;

use panic_halt as _;
use embedded_hal::digital::OutputPin;
use rp2040_hal::{pac, sio::Sio, watchdog::Watchdog, entry, clocks::{Clock, init_clocks_and_plls}};
use rp2040_hal::gpio::{
    Pin, bank0::Gpio15,
    FunctionSio, SioOutput,
    PullDown
};
use embedded_alloc::LlffHeap as Heap;

#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[global_allocator]
static HEAP: Heap = Heap::empty();

static ROM: &[u8] = include_bytes!("cpu_instrs.gb");


fn main_loop(
    mut cpu: Cpu,
    mut mem: MemoryBus,
    mut debug_led: Pin<Gpio15, FunctionSio<SioOutput>, PullDown>,
    mut delay: cortex_m::delay::Delay,
) -> ! {
    blink(&mut delay, &mut debug_led, 2000);
    let mut on = false;
    loop {
        let cycles = cpu.step(&mut mem);
        mem.step(cycles);
        // for _ in 0..cycles {
        //     blink(&mut delay, &mut debug_led, 100);
        // }
        let _ = mem.apu.drain_samples();
        if mem.ppu.frame_ready {
            mem.ppu.frame_ready = false;
            on = !on;
            if on { debug_led.set_high().unwrap(); }
            else { debug_led.set_low().unwrap(); }


            // blink(&mut delay, &mut debug_led, 100);
        }
    }
}

fn blink(delay: &mut cortex_m::delay::Delay,
    led: &mut Pin<Gpio15, FunctionSio<SioOutput>, PullDown>,
    ms: u32) {
    led.set_high().unwrap();
    delay.delay_ms(ms);
    led.set_low().unwrap();
    delay.delay_ms(ms);
}

#[entry]
fn main() -> ! {
    unsafe {
        embedded_alloc::init!(HEAP, 65536);
    }
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let sio = Sio::new(pac.SIO);
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut led = pins.gpio15.into_push_pull_output();

    for _ in 0..3 {
        blink(&mut delay, &mut led, 1000);
    }

    let cpu = Cpu::new();
    blink(&mut delay, &mut led, 1000);
    let mem = MemoryBus::from_static(ROM);
    blink(&mut delay, &mut led, 1000);
    main_loop(cpu, mem, led, delay);
}
