pub struct Timer {
    internal_timer: u16,
    tac: u8,
    tma: u8,
    tima: u8,

    need_to_update_tima: bool,
    write_to_tima: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            internal_timer: 0xABCC,
            tac: 0,
            tma: 0,
            tima: 0,
            need_to_update_tima: false,
            write_to_tima: false,
        }
    }
}

impl Timer {
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xff04 => self.get_div(),
            0xff05 => self.tima,
            0xff06 => self.tma,
            0xff07 => self.tac,
            _ => unreachable!()
        }
    }

    // NOTE: manca il glitch del falling edge quando si scrive su TAC. 
    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xff04 => self.internal_timer = 0,
            0xff05 => {
                self.write_to_tima = true;
                self.tima = data;
            },
            0xff06 => self.tma = data,
            0xff07 => self.tac = data,
            _ => unreachable!()
        }
    }

    pub fn step(&mut self, mcycles: u8) -> bool {
        let mut request_interrupt = false;
        if self.need_to_update_tima && !self.write_to_tima {
            self.tima = self.tma;
            request_interrupt = true;
        }
        self.need_to_update_tima = false;
        self.write_to_tima = false;

        let tcycles = mcycles*4;
        let old_edge = self.get_timer_edge();
        self.internal_timer = self.internal_timer.wrapping_add(tcycles as u16);
        let edge = self.get_timer_edge();
        let timer_tick = old_edge && !edge;
        if timer_tick {
            let tima_falling_edge;
            (self.tima, tima_falling_edge) = self.tima.overflowing_add(1);
            if tima_falling_edge {
                if mcycles == 1 {
                    self.need_to_update_tima = true;
                } else {
                    self.tima = self.tma;
                    request_interrupt = true;
                }
            }
        }
        request_interrupt
    }

    fn get_div(&self) -> u8 {
        ((self.internal_timer & 0xff00) >> 8) as u8
    }

    fn get_timer_edge(&self) -> bool {
        (self.tac & 4) > 0 && match self.tac & 0b11 {
            0 => (self.internal_timer & 0x0200) > 0,
            1 => (self.internal_timer & 0x0008) > 0,
            2 => (self.internal_timer & 0x0020) > 0,
            3 => (self.internal_timer & 0x0080) > 0,
            _ => unreachable!(),
        }
    }

}
