pub struct Timers {

    tima: u8,
    pub tma: u8,
    pub tac: u8,

    pub tima_reload_counter: Option<u8>,

    last_and_value: bool,

}

impl Timers {
    pub fn new() -> Self {
        Self {
            tima: 0,
            tma: 0,
            tac: 0,
            tima_reload_counter: None,
            last_and_value: false,
        }
    }

    pub fn tima(&self) -> u8 {
        self.tima
    }

    pub fn set_tima(&mut self, tima: u8) {
        if self.tima_reload_counter == Some(0) {
            return;
        }
        // Otherwise the write wins and cancels any pending reload/interrupt.
        self.tima = tima;
        self.tima_reload_counter = None;
    }

    pub fn increment_tima(&mut self, div_register: u16) -> bool {
        let mut request_interrupt = false;
        let mut reloaded = false;

        match self.tima_reload_counter {
            Some(0) => {
                self.tima = self.tma;
                self.tima_reload_counter = None;
                request_interrupt = true;
                reloaded = true;
            }
            Some(n) => self.tima_reload_counter = Some(n - 1),
            None => (),
        }

        let new_and_value = Self::compute_and_value(div_register, self.tac);
        let falling_edge = self.last_and_value && !new_and_value;
        self.last_and_value = new_and_value;

        if falling_edge && !reloaded && self.tima_reload_counter.is_none() {
            let (result, overflow) = self.tima.overflowing_add(1);
            self.tima = result;
            if overflow {
                self.tima_reload_counter = Some(3);
            }
        }

        request_interrupt
    }

    fn compute_and_value(div: u16, tac: u8) -> bool {
        let bit_pos= match tac & 0b11 {
            0b00 => 9,
            0b01 => 3,
            0b10 => 5,
            0b11 => 7,
            _ => unreachable!()
        };

        let div_bit = div & (1 << bit_pos) > 0;
        let tac_enabled = tac & 0b100 > 0;

        div_bit && tac_enabled
    } 
}