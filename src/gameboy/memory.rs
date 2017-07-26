use byteorder::{ByteOrder, LittleEndian};

pub struct Memory {
    ram: [u8; 8192],
}

impl Memory {
    pub fn new() -> Memory {
        Memory { ram: [0; 8192] }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        self.ram[addr]
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        let addr = addr as usize;
        LittleEndian::read_u16(&self.ram[addr..])
    }

    pub fn write_u8(&mut self, addr: u16, value: u8) {
        self.ram[addr as usize] = value;
    }

    pub fn write_u16(&mut self, addr: u16, value: u16) {
        let addr = addr as usize;
        LittleEndian::write_u16(&mut self.ram[addr..], value);
    }
}