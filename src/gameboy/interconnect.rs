
use gameboy::{Gfx, Memory};
use super::memory_map::{self, Address};

pub struct Interconnect {
    pub gfx: Gfx,
    pub ram: Memory,
    pub ram_shadow: Memory,
}

impl Interconnect {
    pub fn new() -> Interconnect {
        Interconnect {
            gfx: Gfx::new(),
            ram: Memory::new(),
            ram_shadow: Memory::new(),
        }
    }

    pub fn write_byte(&mut self, addr: u16, byte: u8) {
        match memory_map::map_address(addr) {
            Address::Gfx(value) => self.gfx.write_byte(value, byte),
            _ => {
                panic!("Unable to write byte to: {:#X}, invalid memory region.",
                       addr)
            }
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match memory_map::map_address(addr) {
            Address::Ram(addr) |
            Address::RamShadow(addr) => self.ram.read_u8(addr),
            Address::Gfx(value) => self.gfx.read_byte(value),
            _ => panic!("Unable to read address: {:#X}", addr),
        }
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        match memory_map::map_address(addr) {
            Address::Ram(addr) |
            Address::RamShadow(addr) => self.ram.read_u16(addr),
            Address::Gfx(value) => self.gfx.read_word(value),
            _ => panic!("Unable to read address: {:#X}", addr),
        }
    }
}
