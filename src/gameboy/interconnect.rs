
use gameboy::Gfx;
use super::memory_map::{self, Address};

pub struct Interconnect {
    pub gfx: Gfx,
}

impl Interconnect {
    pub fn new() -> Interconnect {
        Interconnect { gfx: Gfx::new() }
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

    pub fn read_word(&self, addr: u16) -> u16 {
        match memory_map::map_address(addr) {
            Address::Gfx(value) => self.gfx.read_word(value),
            _ => panic!("Unable to read address: {:#X}", addr),
        }
    }
}
