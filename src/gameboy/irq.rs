
pub struct Irq {
    pub request_flag: u8,
    pub enable_flag: u8,
}

impl Irq {
    pub fn new() -> Irq {
        Irq {
            request_flag: 0x00,
            enable_flag: 0x00,
        }
    }

    pub fn request(&mut self, int: Interrupt) {
        use self::Interrupt::*;

        match int {
            Vblank => self.request_flag |= 0x01,
            Lcd => self.request_flag |= 0x02,
            Timer => self.request_flag |= 0x04,
            Serial => self.request_flag |= 0x08,
            _ => panic!("err: unsupported interrupt"),
        }
    }

    pub fn disable(&mut self, int: Interrupt) {
        use self::Interrupt::*;

        match int {
            Vblank => self.request_flag &= !0x01,
            Lcd => self.request_flag &= !0x02,
            Timer => self.request_flag &= !0x04,
            Serial => self.request_flag &= !0x08,
            _ => panic!("err: unsupported interrupt"),
        }
    }

    pub fn enabled(&mut self, int: Interrupt) -> bool {
        use self::Interrupt::*;

        match int {
            Vblank => self.enable_flag & 0x01 == 0x01,
            Lcd => self.enable_flag & 0x02 == 0x02,
            Timer => self.enable_flag & 0x04 == 0x04,
            Serial => self.enable_flag & 0x08 == 0x08,
            _ => panic!("err: unsupported interrupt"),
        }
    }
}

pub enum Interrupt {
    Vblank,
    Lcd,
    Timer,
    Serial,
}
