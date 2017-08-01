use std::ops::Range;

use byteorder::{ByteOrder, LittleEndian};

use gameboy::{Gpu, Memory, Timer};
use gameboy::cartridge::{Cartridge, CartridgeDetails};
use super::memory_map::{self, Address};

const MAIN_MEM_SIZE: usize = 0x2000;
const ZRAM_SIZE: usize = 0x7F;
const MMAP_SIZE: usize = 0x7F;

pub struct Interconnect {
    pub gpu: Gpu,
    pub ram: Memory,
    pub zram: Memory,
    pub mmap_io: Memory,
    pub timer: Timer,
    pub cart: Option<Cartridge>,
    pub interrupt: u8,
}

impl Interconnect {
    pub fn new() -> Interconnect {
        Interconnect {
            gpu: Gpu::new(),
            ram: Memory::new(MAIN_MEM_SIZE),
            zram: Memory::new(ZRAM_SIZE),
            mmap_io: Memory::new(MMAP_SIZE),
            timer: Timer::new(),
            cart: None,
            interrupt: 0x00,
        }
    }

    pub fn with_cart(cart: Cartridge) -> Interconnect {
        Interconnect {
            gpu: Gpu::new(),
            ram: Memory::new(MAIN_MEM_SIZE),
            zram: Memory::new(ZRAM_SIZE),
            mmap_io: Memory::new(MMAP_SIZE),
            timer: Timer::new(),
            cart: Some(cart),
            interrupt: 0x00,
        }
    }

    pub fn step(&mut self, cycles: u8) {
        self.gpu.step(cycles);
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
            Address::Gfx(a) => self.gpu.ram.write_u8(a, byte),
            Address::CartRam(a) => cart.ram.write_u8(a, byte),
            Address::CartRom(a) => cart.rom.write_u8(a, byte),
            Address::ZRam(a) => self.zram.write_u8(a, byte),
            Address::Io(a) => {
                match a {
                    0x04 => self.timer.write_u8(a, byte),
                    0x44 => self.gpu.write_u8(a, byte),
                    _ => panic!("write memory mapped I/O in unsupported range: {:04X}", a),
                }
            }
            Address::InterruptEnableRegister(a) => self.interrupt = byte,
            _ => {
                panic!(
                    "Unable to write byte to: {:#X}, invalid memory region.",
                    addr
                )
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
            Address::Gfx(value) => self.gpu.read_u8(value),
            Address::CartRam(a) => cart.ram.read_u8(a),
            Address::ZRam(a) => self.zram.read_u8(a),
            Address::Io(a) => {
                match a {
                    0x04 => self.timer.read_u8(a), // $FF04 - DIV register
                    0x44 => self.gpu.read_u8(a), // LY $FF44 register in GPU
                    _ => panic!("read memory mapped I/O in unsupported range"),
                }
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
            Address::Gfx(_) => self.gpu.ram.read_bytes(r),
            Address::CartRam(_) => cart.ram.read_bytes(r),
            Address::ZRam(_) => self.zram.read_bytes(r),
            Address::Io(a) => self.mmap_io.read_bytes(r),
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
            Address::Gfx(value) => self.gpu.ram.read_u16(value),
            Address::CartRam(a) => cart.ram.read_u16(a),
            Address::ZRam(a) => self.zram.read_u16(a),
            Address::Io(a) => self.mmap_io.read_u16(a),
            _ => panic!("Unable to read address: {:#X}", addr),
        }
    }
}
