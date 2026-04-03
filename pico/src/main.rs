#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;
use rp2040_hal::entry;

#[entry]
fn main() -> ! {
    loop {}
}
