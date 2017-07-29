use std::ops::Range;

use byteorder::{ByteOrder, LittleEndian};

use gameboy::{Gfx, Memory};
use gameboy::cartridge::{Cartridge, CartridgeDetails};
use super::memory_map::{self, Address};

const MAIN_MEM_SIZE: usize = 0x2000;
const ZRAM_SIZE: usize = 0x7F;

pub struct Interconnect {
    pub gfx: Gfx,
    pub ram: Memory,
    pub zram: Memory,
    pub cart: Option<Cartridge>,
    pub interrupt: u8,
}

impl Interconnect {
    pub fn new() -> Interconnect {
        Interconnect {
            gfx: Gfx::new(),
            ram: Memory::new(MAIN_MEM_SIZE),
            zram: Memory::new(ZRAM_SIZE),
            cart: None,
            interrupt: 0x00,
        }
    }

    pub fn with_cart(cart: Cartridge) -> Interconnect {
        Interconnect {
            gfx: Gfx::new(),
            ram: Memory::new(MAIN_MEM_SIZE),
            zram: Memory::new(ZRAM_SIZE),
            cart: Some(cart),
            interrupt: 0x00,
        }
    }

    pub fn cart_details(&self) -> CartridgeDetails {
        let cart = self.cart.as_ref().expect("Cartridge is empty");

        cart.details(&self)
    }

    pub fn write_u8(&mut self, addr: u16, byte: u8) {
        let cart = self.cart.as_mut().expect("Cartridge is empty");

        match memory_map::map_address(addr) {
            Address::Ram(a) |
            Address::RamShadow(a) => self.ram.write_u8(a, byte),
            Address::Gfx(a) => self.gfx.write_u8(a, byte),
            Address::CartRam(a) => cart.ram.write_u8(a, byte),
            Address::CartRom(a) => cart.rom.write_u8(a, byte),
            Address::ZRam(a) => self.zram.write_u8(a, byte),
            Address::Io(a) => {
                println!("err: tried to write to memory mapped I/O at {:04X} (not implemented yet)",
                         a)
            }
            Address::InterruptEnableRegister(a) => self.interrupt = byte,
            _ => {
                panic!("Unable to write byte to: {:#X}, invalid memory region.",
                       addr)
            }
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        let cart = self.cart.as_ref().expect("Cartridge is empty");

        match memory_map::map_address(addr) {
            Address::Ram(addr) |
            Address::RamShadow(addr) => self.ram.read_u8(addr),
            Address::CartRom(addr) |
            Address::CartRomOtherBank(addr) => cart.rom.read_u8(addr),
            Address::Gfx(value) => self.gfx.read_u8(value),
            Address::CartRam(a) => cart.ram.read_u8(a),
            Address::ZRam(a) => self.zram.read_u8(a),
            Address::Io(a) => {
                println!("err: tried to read from memory mapped I/O (not implemented yet)");
                0
            }
            Address::InterruptEnableRegister(a) => self.interrupt,
            _ => panic!("Unable to read address: {:#X}", addr),
        }
    }

    pub fn read_bytes(&self, r: Range<u16>) -> &[u8] {
        let cart = self.cart.as_ref().expect("Cartridge is empty");

        match memory_map::map_address(r.start) {
            Address::Ram(_) |
            Address::RamShadow(_) => self.ram.read_bytes(r),
            Address::CartRom(_) |
            Address::CartRomOtherBank(_) => cart.rom.read_bytes(r),
            Address::Gfx(_) => self.gfx.read_bytes(r),
            Address::CartRam(_) => cart.ram.read_bytes(r),
            Address::ZRam(_) => self.zram.read_bytes(r),
            Address::Io(a) => {
                println!("err: tried to read from memory mapped I/O (not implemented yet)");
                &[]
            }
            _ => panic!("Unable to read address range: {:?}", r),
        }
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        let cart = self.cart.as_ref().expect("Cartridge is empty");

        match memory_map::map_address(addr) {
            Address::Ram(addr) |
            Address::RamShadow(addr) => self.ram.read_u16(addr),
            Address::CartRom(addr) |
            Address::CartRomOtherBank(addr) => cart.rom.read_u16(addr),
            Address::Gfx(value) => self.gfx.read_u16(value),
            Address::CartRam(a) => cart.ram.read_u16(a),
            Address::ZRam(a) => self.zram.read_u16(a),
            Address::Io(a) => {
                println!("err: tried to read from memory mapped I/O (not implemented yet)");
                0
            }
            _ => panic!("Unable to read address: {:#X}", addr),
        }
    }
}
