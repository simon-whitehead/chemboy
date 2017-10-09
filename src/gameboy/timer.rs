use gameboy::{MAX_CPU_CYCLES, Interconnect, Irq, Interrupt};

pub struct Timer {
    div: usize,
    tima: u8,
    tma: u8,
    tac: u8,
    cycles: usize,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            div: 0x00,
            tima: 0x00,
            tma: 0x00,
            tac: 0x00,
            cycles: MAX_CPU_CYCLES / 0x1000,
        }
    }

    pub fn reset(&mut self) {
        *self = Timer::new();
    }

    pub fn step(&mut self, irq: &mut Irq, cycles: usize) -> Result<(), String> {
        // DIV register increments regardless
        self.inc_div_register(cycles);

        if !self.enabled() {
            return Ok(());
        }

        self.inc_tima_register(irq, cycles);

        Ok(())
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0x04 => self.div as u8,
            0x05 => self.tima,
            0x06 => self.tma,
            0x07 => self.tac,
            _ => panic!("read timer memory that is unmapped"),
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            0x04 => self.div = 0,
            0x05 => self.tima = val,
            0x06 => self.tma = val,
            0x07 => self.tac = val,
            _ => panic!("read timer memory that is unmapped"),
        }
    }

    fn enabled(&self) -> bool {
        self.tac & 0x04 == 0x04
    }

    fn inc_div_register(&mut self, cycles: usize) {
        self.div += cycles;
        if self.div >= 0xFF {
            self.div = 0x00;
        }
    }

    fn inc_tima_register(&mut self, irq: &mut Irq, cycles: usize) {
        if self.enabled() {
            if self.cycles < cycles {
                self.set_timer_frequency();
                if self.tima == 0xFF {
                    self.tima = self.tma; // set the TIMA register to be whatever is in the modulo TMA register
                    irq.request(Interrupt::Timer); // it overflowed, request a timer interrupt
                }
                self.tima.wrapping_add(0x01);
            }
            self.cycles = self.cycles.wrapping_sub(cycles);
        }
    }

    fn set_timer_frequency(&mut self) {
        let speed = self.tac & 0x03;
        self.cycles = match speed {
            0x00 => MAX_CPU_CYCLES / 0x1000,
            0x01 => MAX_CPU_CYCLES / 0x40000,
            0x02 => MAX_CPU_CYCLES / 0x10000,
            0x03 => MAX_CPU_CYCLES / 0x4000,
            _ => 0,
        };
    }
}
