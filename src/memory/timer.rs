const DIV_ADDR: u16 = 0xff04;
const TIMA_ADDR: u16 = 0xff05;
const TMA_ADDR: u16 = 0xff06;
const TAC_ADDR: u16 = 0xff07;

pub struct Timer {
    internal_timer: u16,
    tac: u8,
    tma: u8,
    tima: u8,

    overflow_pending: bool,
    tima_written_this_mcycle: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            internal_timer: 0xABCC,
            tac: 0,
            tma: 0,
            tima: 0,
            overflow_pending: false,
            tima_written_this_mcycle: false,
        }
    }
}

impl Timer {
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            DIV_ADDR => self.get_div(),
            TIMA_ADDR => self.tima,
            TMA_ADDR => self.tma,
            TAC_ADDR => self.tac,
            _ => unreachable!(),
        }
    }

    // NOTE: manca il glitch del falling edge quando si scrive su TAC.
    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            DIV_ADDR => {
                let old_edge = self.get_timer_edge();
                self.internal_timer = 0;
                if old_edge {
                    self.apply_timer_tick();
                }
            }
            TIMA_ADDR => {
                self.tima_written_this_mcycle = true;
                self.tima = data;
            }
            TMA_ADDR => self.tma = data,
            TAC_ADDR => {
                let old_edge = self.get_timer_edge();
                self.tac = data;
                if old_edge && !self.get_timer_edge() {
                    self.apply_timer_tick();
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn step(&mut self, mcycles: u8) -> bool {
        let mut req_int = false;
        for _ in 0..mcycles {
            req_int |= self.step_mcycle();
        }
        req_int
    }

    pub fn step_mcycle(&mut self) -> bool {
        let mut request_interrupt = false;

        if self.overflow_pending {
            if !self.tima_written_this_mcycle {
                self.tima = self.tma;
                request_interrupt = true;
            }
            self.overflow_pending = false;
        }

        self.tima_written_this_mcycle = false;

        let old_edge = self.get_timer_edge();
        self.internal_timer = self.internal_timer.wrapping_add(4);
        let edge = self.get_timer_edge();

        let timer_tick = old_edge && !edge;
        if timer_tick {
            self.apply_timer_tick();
        }
        request_interrupt
    }

    fn get_div(&self) -> u8 {
        ((self.internal_timer & 0xff00) >> 8) as u8
    }

    fn get_timer_edge(&self) -> bool {
        (self.tac & 4) > 0
            && match self.tac & 0b11 {
                0 => (self.internal_timer & 0x0200) > 0,
                1 => (self.internal_timer & 0x0008) > 0,
                2 => (self.internal_timer & 0x0020) > 0,
                3 => (self.internal_timer & 0x0080) > 0,
                _ => unreachable!(),
            }
    }

    fn apply_timer_tick(&mut self) {
        let (new_tima, overflow) = self.tima.overflowing_add(1);
        self.tima = new_tima;
        if overflow {
            self.overflow_pending = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn div_initial_value() {
        let timer = Timer::default();
        assert_eq!(timer.read(DIV_ADDR), 0xAB);
    }

    #[test]
    fn div_resets_on_write() {
        let mut timer = Timer::default();
        timer.write(DIV_ADDR, 0xFF);
        assert_eq!(timer.read(DIV_ADDR), 0x00);
    }

    #[test]
    fn div_increments_over_time() {
        let mut timer = Timer::default();
        timer.write(DIV_ADDR, 0x00);
        for _ in 0..64 {
            timer.step(1);
        }
        assert_eq!(timer.read(DIV_ADDR), 0x01);
    }

    #[test]
    fn tac_read_write() {
        let mut timer = Timer::default();
        timer.write(TAC_ADDR, 0b101);
        assert_eq!(timer.read(TAC_ADDR), 0b101);
    }

    #[test]
    fn tma_read_write() {
        let mut timer = Timer::default();
        timer.write(TMA_ADDR, 0x42);
        assert_eq!(timer.read(TMA_ADDR), 0x42);
    }

    #[test]
    fn tima_read_write() {
        let mut timer = Timer::default();
        timer.write(TIMA_ADDR, 0x10);
        assert_eq!(timer.read(TIMA_ADDR), 0x10);
    }

    #[test]
    fn tima_does_not_tick_when_disabled() {
        let mut timer = Timer::default();
        timer.write(DIV_ADDR, 0x00);
        timer.write(TAC_ADDR, 0x00);
        for _ in 0..1000 {
            timer.step(1);
        }
        assert_eq!(timer.read(TIMA_ADDR), 0x00);
    }

    #[test]
    fn tima_ticks_at_4096hz_mode0() {
        let mut timer = Timer::default();
        timer.write(DIV_ADDR, 0x00);
        timer.write(TAC_ADDR, 0b100);
        for _ in 0..64 {
            timer.step(4);
        }
        assert_eq!(timer.read(TIMA_ADDR), 1);
    }

    #[test]
    fn tima_ticks_at_262144hz_mode1() {
        let mut timer = Timer::default();
        timer.write(DIV_ADDR, 0x00);
        timer.write(TAC_ADDR, 0b101);
        for _ in 0..2 {
            timer.step(2);
        }
        assert_eq!(timer.read(TIMA_ADDR), 1);
    }

    #[test]
    fn tima_overflow_requests_interrupt() {
        let mut timer = Timer::default();
        timer.write(DIV_ADDR, 0x00);
        timer.write(TIMA_ADDR, 0xFF);
        timer.write(TAC_ADDR, 0b101);
        let mut interrupt = false;
        for _ in 0..10 {
            interrupt |= timer.step(1);
        }
        assert!(interrupt);
    }

    #[test]
    fn tima_reloads_tma_after_overflow() {
        let mut timer = Timer::default();
        timer.write(DIV_ADDR, 0x00);
        timer.write(TMA_ADDR, 0x30);
        timer.write(TIMA_ADDR, 0xFF);
        timer.write(TAC_ADDR, 0b101);
        for _ in 0..10 {
            timer.step(1);
        }
        assert_eq!(timer.read(TIMA_ADDR), 0x31);
    }

    #[test]
    fn tima_write_cancels_tma_reload() {
        let mut timer = Timer::default();
        timer.write(DIV_ADDR, 0x00);
        timer.write(TMA_ADDR, 0x30);
        timer.write(TIMA_ADDR, 0xFF);
        timer.write(TAC_ADDR, 0b101);
        timer.step(1);
        timer.step(1);
        timer.write(TIMA_ADDR, 0x42);
        timer.step(1);
        assert_eq!(timer.read(TIMA_ADDR), 0x42);
    }
}
