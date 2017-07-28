use std;
use std::fs::File;
use std::io::Write;
use std::ops::{Deref, Range};
use std::path::Path;

use byteorder::{ByteOrder, LittleEndian};

pub struct Memory {
    ram: Vec<u8>,
}

impl Memory {
    pub fn new(capacity: usize) -> Memory {
        Memory { ram: vec![0x00; capacity] }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        self.ram[addr]
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        let addr = addr as usize;
        LittleEndian::read_u16(&self.ram[addr..])
    }

    pub fn read_bytes(&self, r: Range<u16>) -> &[u8] {
        &self.ram[r.start as usize..r.end as usize]
    }

    pub fn write_u8(&mut self, addr: u16, value: u8) {
        self.ram[addr as usize] = value;
    }

    pub fn write_u16(&mut self, addr: u16, value: u16) {
        let addr = addr as usize;
        LittleEndian::write_u16(&mut self.ram[addr..], value);
    }

    pub fn write_bytes(&mut self, addr: u16, bytes: &[u8]) {
        let mut addr = addr as usize;

        for b in bytes {
            self.ram[addr] = *b;
            addr += 1;
        }
    }

    pub fn dump<P>(&self, p: P) -> Result<(), std::io::Error>
        where P: AsRef<Path>
    {
        let mut f = File::create(p)?;

        f.write_all(&self.ram)
    }
}

impl Deref for Memory {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.ram
    }
}