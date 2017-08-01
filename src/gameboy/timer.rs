
pub struct Timer {
    div: u8,
}

impl Timer {
    pub fn new() -> Timer {
        Timer { div: 0x00 }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0x04 => self.div,
            _ => panic!("read timer memory that is unmapped"),
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            0x04 => self.div = val,
            _ => panic!("read timer memory that is unmapped"),
        }
    }
}
