#[derive(Eq, PartialEq)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    pub pc: u16,
    pub sp: usize,

    pub div: u8,

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
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,

            pc: 0x00,
            sp: 0xFFFE,

            div: 0x00,

            flags: Flags::new(),
        }
    }

    pub fn set_pc(&mut self, addr: u16) {
        self.pc = addr;
    }

    pub fn get_af(&self) -> u16 {
        ((self.a as u16) << 0x08) | self.flags.to_byte() as u16
    }

    pub fn set_af(&mut self, val: u16) {
        self.a = (val >> 0x08) as u8;
        self.flags = Flags::from_byte((val & 0xF0) as u8);
    }

    pub fn get_bc(&self) -> u16 {
        ((self.b as u16) << 0x08) | self.c as u16
    }

    pub fn set_bc(&mut self, val: u16) {
        self.b = (val >> 0x08) as u8;
        self.c = (val & 0xFF) as u8;
    }

    pub fn get_de(&self) -> u16 {
        ((self.d as u16) << 0x08) | self.e as u16
    }

    pub fn set_de(&mut self, val: u16) {
        self.d = (val >> 0x08) as u8;
        self.e = (val & 0xFF) as u8;
    }

    pub fn get_hl(&self) -> u16 {
        ((self.h as u16) << 0x08) | self.l as u16
    }

    pub fn set_hl(&mut self, val: u16) {
        self.h = (val >> 0x08) as u8;
        self.l = (val & 0xFF) as u8;
    }

    pub fn dump(&self) {
        println!("AF: {:02X}", self.get_af());
        println!("BC: {:02X}", self.get_bc());
        println!("DE: {:02X}", self.get_de());;
        println!("HL: {:02X}", self.get_hl());;
    }
}

#[derive(Eq, PartialEq)]
pub struct Flags {
    pub zero: bool,
    pub negative: bool,
    pub half_carry: bool,
    pub carry: bool,
}

impl Flags {
    pub fn new() -> Flags {
        Flags {
            zero: true,
            negative: false,
            half_carry: false,
            carry: false,
        }
    }

    pub fn from_byte(b: u8) -> Flags {
        Flags {
            zero: b & 0x80 == 0x80,
            negative: b & 0x40 == 0x40,
            half_carry: b & 0x20 == 0x20,
            carry: b & 0x10 == 0x10,
        }
    }

    pub fn to_byte(&self) -> u8 {
        0x00 | (if self.zero { 0x80 } else { 0x00 }) | (if self.negative { 0x40 } else { 0x00 }) |
        (if self.half_carry { 0x20 } else { 0x00 }) | (if self.carry { 0x10 } else { 0x00 })
    }

    pub fn dump(&self) {
        println!("Z: {}", self.zero);
        println!("N: {}", self.negative);
        println!("H: {}", self.half_carry);
        println!("C: {}", self.carry);
    }
}
