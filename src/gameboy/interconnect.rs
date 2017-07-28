use std::ops::Range;

use byteorder::{ByteOrder, LittleEndian};

use gameboy::{Cartridge, CartridgeDetails, Gfx, Memory};
use super::memory_map::{self, Address};

pub struct Interconnect {
    pub gfx: Gfx,
    pub ram: Memory,
    pub cart: Option<Cartridge>,
}

impl Interconnect {
    pub fn new() -> Interconnect {
        Interconnect {
            gfx: Gfx::new(),
            ram: Memory::new(),
            cart: None,
        }
    }

    pub fn with_cart(cart: Cartridge) -> Interconnect {
        Interconnect {
            gfx: Gfx::new(),
            ram: Memory::new(),
            cart: Some(cart),
        }
    }

    pub fn cart_details(&self) -> CartridgeDetails {
        let cart = self.cart.as_ref().expect("Cartridge is empty");

        cart.details(&self)
    }

    pub fn write_byte(&mut self, addr: u16, byte: u8) {
        match memory_map::map_address(addr) {
            Address::Ram(a) |
            Address::RamShadow(a) => self.ram.write_u8(a, byte),
            Address::Gfx(value) => self.gfx.write_u8(value, byte),
            _ => {
                panic!(
                    "Unable to write byte to: {:#X}, invalid memory region.",
                    addr
                )
            }
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        let cart = self.cart.as_ref().expect("Cartridge is empty");

        match memory_map::map_address(addr) {
            Address::Ram(addr) |
            Address::RamShadow(addr) => self.ram.read_u8(addr),
            Address::CartRom(addr) |
            Address::CartRomOtherBank(addr) => cart[addr as usize],
            Address::Gfx(value) => self.gfx.read_u8(value),
            _ => panic!("Unable to read address: {:#X}", addr),
        }
    }

    pub fn read_bytes(&self, r: Range<u16>) -> &[u8] {
        let cart = self.cart.as_ref().expect("Cartridge is empty");

        match memory_map::map_address(r.start) {
            Address::Ram(addr) |
            Address::RamShadow(addr) => self.ram.read_bytes(r),
            Address::CartRom(addr) |
            Address::CartRomOtherBank(addr) => &cart[r.start as usize..r.end as usize],
            Address::Gfx(value) => self.gfx.read_bytes(r),
            _ => panic!("Unable to read address range: {:?}", r),
        }
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        let cart = self.cart.as_ref().expect("Cartridge is empty");

        match memory_map::map_address(addr) {
            Address::Ram(addr) |
            Address::RamShadow(addr) => self.ram.read_u16(addr),
            Address::CartRom(addr) |
            Address::CartRomOtherBank(addr) => LittleEndian::read_u16(&cart[addr as usize..]),
            Address::Gfx(value) => self.gfx.read_u16(value),
            _ => panic!("Unable to read address: {:#X}", addr),
        }
    }
}
