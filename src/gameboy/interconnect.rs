use byteorder::{ByteOrder, LittleEndian};

use gameboy::{Gfx, Memory};
use super::memory_map::{self, Address};

pub struct Interconnect {
    pub gfx: Gfx,
    pub ram: Memory,
    pub rom: Box<[u8]>,
}

impl Interconnect {
    pub fn new() -> Interconnect {
        Interconnect {
            gfx: Gfx::new(),
            ram: Memory::new(),
            rom: vec![].into_boxed_slice(),
        }
    }

    pub fn with_rom(rom: Box<[u8]>) -> Interconnect {
        Interconnect {
            gfx: Gfx::new(),
            ram: Memory::new(),
            rom: rom,
        }
    }

    pub fn write_byte(&mut self, addr: u16, byte: u8) {
        match memory_map::map_address(addr) {
            Address::Ram(a) |
            Address::RamShadow(a) => self.ram.write_u8(a, byte),
            Address::Gfx(value) => self.gfx.write_u8(value, byte),
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
            Address::CartRom(addr) |
            Address::CartRomOtherBank(addr) => self.rom[addr as usize],
            Address::Gfx(value) => self.gfx.read_u8(value),
            _ => panic!("Unable to read address: {:#X}", addr),
        }
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        match memory_map::map_address(addr) {
            Address::Ram(addr) |
            Address::RamShadow(addr) => self.ram.read_u16(addr),
            Address::CartRom(addr) |
            Address::CartRomOtherBank(addr) => LittleEndian::read_u16(&self.rom[addr as usize..]),
            Address::Gfx(value) => self.gfx.read_u16(value),
            _ => panic!("Unable to read address: {:#X}", addr),
        }
    }
}
