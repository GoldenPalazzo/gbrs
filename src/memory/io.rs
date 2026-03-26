pub const FLAG_TRANSFER: u8 = 0x80;
pub const FLAG_HISPEED: u8 = 0x02;
pub const FLAG_MASTER: u8 = 0x01;

const DATA_ADDR: u16 = 0xff01;
const CTRL_ADDR: u16 = 0xff02;
const SAMPLE_CYCLES: u8 = 10;

#[derive(Default)]
pub struct Serial {
    data: u8,
    control: u8,

    internal_cycles: u8,
}

impl Serial {
    pub fn step(&mut self, cycles: u8) {
        self.internal_cycles = self.internal_cycles.wrapping_add(cycles);
        if self.check_flag(FLAG_TRANSFER)
                && self.internal_cycles >= SAMPLE_CYCLES {
            print!("{:?}", std::char::from_u32(self.data as u32));
            self.set_flag(FLAG_TRANSFER, false);
            self.internal_cycles = 0;
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            DATA_ADDR => {
                self.data = data;
            }
            CTRL_ADDR => {
                self.control = data;
            }
            _ => unreachable!()
        }
    }

    fn check_flag(&self, flag: u8) -> bool { (self.control & flag) != 0 }
    fn set_flag(&mut self, flag: u8, value: bool) {
        if value { self.control |= flag; }
        else { self.control &= !flag; }
    }
}
