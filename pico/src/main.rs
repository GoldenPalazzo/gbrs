#![no_std]
#![no_main]
use gbrs_engine::cpu::cpu::Cpu;
use gbrs_engine::memory::bus::MemoryBus;

use panic_halt as _;
use embedded_hal::digital::OutputPin;
use rp2040_hal::{Timer, clocks::{Clock, ClocksManager, init_clocks_and_plls}, entry, fugit::RateExtU32, pac, pll::{common_configs::PLL_USB_48MHZ, setup_pll_blocking}, sio::Sio, watchdog::Watchdog, xosc::setup_xosc_blocking};
use rp2040_hal::gpio::{
    Pin, bank0::Gpio15,
    FunctionSio, SioOutput,
    PullDown
};
use rp2040_hal::pll::PLLConfig;
use embedded_alloc::LlffHeap as Heap;

#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[global_allocator]
static HEAP: Heap = Heap::empty();

static ROM: &[u8] = include_bytes!("cpu_instrs.gb");


fn main_loop_dbg(
    mut cpu: Cpu,
    mut mem: MemoryBus,
    mut debug_led: Pin<Gpio15, FunctionSio<SioOutput>, PullDown>,
    mut delay: cortex_m::delay::Delay,
    mut timer: Timer
) -> ! {
    let mut cycles_count: u32 = 0;
    let mut cpu_time: u64 = 0;
    // let mut bus_time: u64 = 0;
    let mut other_time: u64 = 0;
    let mut ppu_time: u64 = 0;
    // let mut last_time = timer.get_counter().ticks();
    blink(&mut delay, &mut debug_led, 100);

    loop {
        let t0 = timer.get_counter().ticks();
        let cycles = cpu.step(&mut mem);
        let t1 = timer.get_counter().ticks();
        mem.step_no_ppu(cycles);
        let t2 = timer.get_counter().ticks();
        mem.step_ppu(cycles);
        let t3 = timer.get_counter().ticks();
        cpu_time += t1 - t0;
        // bus_time += t2 - t1;
        other_time += t2 - t1;
        ppu_time += t3 - t2;
        cycles_count += cycles as u32;
        if cycles_count >= 1_048_576 {
            cycles_count -= 1_048_576;
            // let now = timer.get_counter().ticks();
            // let elapsed_us = now - last_time;
            blink(&mut delay, &mut debug_led, (cpu_time / 1000) as u32);
            delay.delay_ms(1000);
            blink(&mut delay, &mut debug_led, (other_time / 1000) as u32);
            delay.delay_ms(1000);
            blink(&mut delay, &mut debug_led, (ppu_time / 1000) as u32);
            delay.delay_ms(2000);
            cpu_time = 0;
            // bus_time = 0;
            other_time = 0;
            ppu_time = 0;
            // last_time = timer.get_counter().ticks();
        }
    }
}

fn main_loop(
    mut cpu: Cpu,
    mut mem: MemoryBus,
    mut debug_led: Pin<Gpio15, FunctionSio<SioOutput>, PullDown>,
    mut delay: cortex_m::delay::Delay,
    timer: Timer
    ) -> ! {
    let mut cycles_count: u32 = 0;

    let mut last_time = timer.get_counter().ticks();
    blink(&mut delay, &mut debug_led, 100);

    loop {
        let cycles = cpu.step(&mut mem);
        mem.step(cycles);
        cycles_count += cycles as u32;
        if cycles_count >= 1_048_576 {
            cycles_count -= 1_048_576;
            let now = timer.get_counter().ticks();
            let elapsed_us = now - last_time;
            blink(&mut delay, &mut debug_led, (elapsed_us / 1000) as u32);
            last_time = timer.get_counter().ticks();
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
    let external_xtal_freq_hz = 12_000_000u32;
    let xosc = setup_xosc_blocking(pac.XOSC, external_xtal_freq_hz.Hz()).unwrap();
    let mut clocks = ClocksManager::new(pac.CLOCKS);
    let pll_sys = setup_pll_blocking(pac.PLL_SYS, xosc.operating_frequency(), PLLConfig {
        vco_freq: 1200_000_000u32.Hz(),
        refdiv: 1,
        post_div1: 3,
        post_div2: 2,
    }, &mut clocks, &mut pac.RESETS).unwrap();
    let pll_usb = setup_pll_blocking(
        pac.PLL_USB,
        xosc.operating_frequency(),
        PLL_USB_48MHZ,
        &mut clocks,
        &mut pac.RESETS
        ).unwrap();
    clocks.init_default(&xosc, &pll_sys, &pll_usb).unwrap();


    let delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let led = pins.gpio15.into_push_pull_output();
    let timer = rp2040_hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let cpu = Cpu::new();
    let mut mem = MemoryBus::from_static(ROM);
    mem.apu.debug_disable = true;
    // mem.ppu.debug_skip_render = true;
    main_loop(cpu, mem, led, delay, timer);
}
