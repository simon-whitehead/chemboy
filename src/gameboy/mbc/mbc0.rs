
// MBC0 - no memory banking

use std::ops::Range;

use gameboy::Memory;
use gameboy::mbc::mbc::get_ram_size;
use gameboy::mbc::MBC;

pub struct MBC0 {
    pub rom: Memory,
    pub ram: Memory,
}

impl MBC0 {
    pub fn new(rom: &[u8]) -> MBC0 {
        let mut r = Memory::new(rom.len());
        r.write_bytes(0x00, rom);

        let ram_size = get_ram_size(rom[0x149]);
        let ram = Memory::new(ram_size);

        MBC0 { rom: r, ram: ram }
    }
}

impl MBC for MBC0 {
    fn read_ram_u8(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn read_rom_u8(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }

    fn read_ram_u16(&self, addr: u16) -> u16 {
        self.ram.read_u16(addr)
    }

    fn read_rom_u16(&self, addr: u16) -> u16 {
        self.rom.read_u16(addr)
    }

    fn write_ram_u8(&mut self, addr: u16, b: u8) {
        ()
    }

    fn write_rom_u8(&mut self, addr: u16, b: u8) {
        ()
    }

    fn write_ram_u16(&mut self, addr: u16, b: u16) {
        self.ram.write_u16(addr, b)
    }
}