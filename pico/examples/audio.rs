#![no_std]
#![no_main]
use core::f32::consts::TAU;
use embedded_alloc::LlffHeap as Heap;
use embedded_hal::digital::OutputPin;
use libm::sinf;
use panic_halt as _;
use rp2040_hal::gpio::{FunctionSio, Pin, PullDown, SioOutput, bank0::Gpio16};
use rp2040_hal::pio::PIOExt;
use rp2040_hal::pll::PLLConfig;
use rp2040_hal::{
    Clock,
    clocks::ClocksManager,
    entry,
    fugit::RateExtU32,
    pac,
    pll::{common_configs::PLL_USB_48MHZ, setup_pll_blocking},
    sio::Sio,
    xosc::setup_xosc_blocking,
};
use rp2040_i2s::{I2SOutput, PioClockDivider};

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
    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut led = pins.gpio16.into_push_pull_output();
    for _ in 0..3 {
        led.set_high().unwrap();
        cortex_m::asm::delay(1_000_000);
        led.set_low().unwrap();
        cortex_m::asm::delay(1_000_000);
    }

    let external_xtal_freq_hz = 12_000_000u32;
    let xosc = setup_xosc_blocking(pac.XOSC, external_xtal_freq_hz.Hz()).unwrap();
    let mut clocks = ClocksManager::new(pac.CLOCKS);
    let pll_sys = setup_pll_blocking(
        pac.PLL_SYS,
        xosc.operating_frequency(),
        PLLConfig {
            vco_freq: 1_228_800_000u32.Hz(),
            refdiv: 1,
            post_div1: 5,
            post_div2: 4,
        },
        &mut clocks,
        &mut pac.RESETS,
    )
    .unwrap();
    let pll_usb = setup_pll_blocking(
        pac.PLL_USB,
        xosc.operating_frequency(),
        PLL_USB_48MHZ,
        &mut clocks,
        &mut pac.RESETS,
    )
    .unwrap();
    clocks.init_default(&xosc, &pll_sys, &pll_usb).unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let blck_pin = pins.gpio12;
    let wsel_pin = pins.gpio13;
    let din_pin = pins.gpio14;
    let (mut pio0, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let i2s = I2SOutput::new(
        &mut pio0,
        PioClockDivider::FromSystemClock(61_440_000u32.Hz()),
        sm0,
        din_pin,
        blck_pin,
        wsel_pin,
    )
    .unwrap();
    let (_sm, _rx, mut tx_fifo) = i2s.split();

    let sample_rate: u32 = 48000;
    let freq: f32 = 440.0;
    let mut phase: f32 = 0.0;
    let phase_step: f32 = TAU * freq / sample_rate as f32;
    for _ in 0..4 {
        while !tx_fifo.write(0x7FFF_7FFF) {}
    }
    let sm = _sm.start();
    loop {
        led.set_high().unwrap();

        // let val = (sinf(phase) * 32767.0) as i16;
        phase += phase_step;
        if phase >= TAU {
            phase -= TAU;
        }

        // let sample = ((val as u16 as u32) << 16) | (val as u16 as u32);
        let sample: u32 = if phase < TAU / 2.0 {
            0x7FFF_0000
        } else {
            0x8000_0000
        };
        while !tx_fifo.write(sample) {}
        while !tx_fifo.write(0) {}
        led.set_low().unwrap();
    }
}
