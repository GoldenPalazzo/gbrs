#![no_std]
#![no_main]

use embedded_alloc::LlffHeap as Heap;
use embedded_hal::digital::OutputPin;
use panic_halt as _;
use rp2040_boot2;
use rp2040_hal::{
    clocks::{Clock, init_clocks_and_plls},
    entry, pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[entry]
fn main() -> ! {
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
    loop {
        led.set_high().unwrap(); // ON
        // cortex_m::asm::delay(5_000_000);
        delay.delay_ms(1000);

        led.set_low().unwrap(); // OFF
        // cortex_m::asm::delay(5_000_000);
        delay.delay_ms(1000);
    }
}
