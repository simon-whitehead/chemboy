pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,

    pub pc: usize,
    pub sp: usize,

    pub flags: Flags,
}

impl Registers {
    pub fn new(gameboy_color: bool) -> Registers {
        // Taken from http://www.devrs.com/gb/files/gbspec.txt
        let a = match gameboy_color {
            true => 0x11,
            false => 0x01,
        };

        Registers {
            a: a,
            f: 0xB0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,

            pc: 0x00,
            sp: 0xFFFE,

            flags: Flags::new(),
        }
    }

    pub fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }
}

pub struct Flags {
    pub zero: bool,
}

impl Flags {
    pub fn new() -> Flags {
        Flags { zero: true }
    }
}
