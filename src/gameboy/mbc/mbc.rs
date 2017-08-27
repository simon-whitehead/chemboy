use std::ops::Range;

use byteorder::{ByteOrder, LittleEndian};

pub trait MBC {
    fn read_ram_u8(&self, addr: u16) -> u8;
    fn read_rom_u8(&self, addr: u16) -> u8;

    fn read_ram_u16(&self, addr: u16) -> u16 {
        let a = self.read_ram_u8(addr);
        let b = self.read_ram_u8(addr + 0x01);

        LittleEndian::read_u16(&[a, b])
    }

    fn read_rom_u16(&self, addr: u16) -> u16 {
        let a = self.read_rom_u8(addr);
        let b = self.read_rom_u8(addr + 0x01);

        LittleEndian::read_u16(&[a, b])
    }

    fn write_ram_u8(&mut self, addr: u16, b: u8);
    fn write_rom_u8(&mut self, addr: u16, b: u8);

    fn write_ram_u16(&mut self, addr: u16, b: u16);
}

pub fn get_ram_size(b: u8) -> usize {
    match b {
        0x01 => 0x800,      // 2KB (2048 bytes)
        0x02 => 0x2000,     // 8KB (8096 bytes)
        0x03 => 0x8000,     // 32KB
        _ => 0x00,
    }
}