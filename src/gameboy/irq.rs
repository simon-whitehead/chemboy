
pub struct Irq {
    request: u8,
    enable: u8,
}

impl Irq {
    pub fn new() -> Irq {
        Irq {
            request: 0x00,
            enable: 0x00,
        }
    }

    pub fn request(&mut self, int: Interrupt) {
        use self::Interrupt::*;

        match int {
            Timer => self.request |= 0x04,
        }
    }

    pub fn unrequest(&mut self, int: Interrupt) {
        use self::Interrupt::*;

        match int {
            Timer => self.request &= !0x04,
        }
    }

    pub fn enable(&mut self, int: Interrupt) {
        use self::Interrupt::*;

        match int {
            Timer => self.enable |= 0x04,
        }
    }

    pub fn disable(&mut self, int: Interrupt) {
        use self::Interrupt::*;

        match int {
            Timer => self.enable &= !0x04,
        }
    }
}

pub enum Interrupt {
    Timer,
}
