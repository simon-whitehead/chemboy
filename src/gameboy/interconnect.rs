use std::ops::Range;

use byteorder::{ByteOrder, LittleEndian};

use gameboy::{CPU_FREQUENCY, MAX_CPU_CYCLES, MAX_DIV_REG_CYCLES, Gpu, Irq, Memory, Timer};
use gameboy::cartridge::{Cartridge, CartridgeDetails};
use gameboy::frame::Frame;
use super::memory_map::{self, Address};

const MAIN_MEM_SIZE: usize = 0x2000;
const ZRAM_SIZE: usize = 0x80;
const MMAP_SIZE: usize = 0x80;

pub struct Interconnect {
    pub gpu: Gpu,
    pub ram: Memory,
    pub zram: Memory,
    pub mmap_io: Memory,
    pub timer: Timer,
    pub irq: Irq,
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
            irq: Irq::new(),
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
            irq: Irq::new(),
            cart: Some(cart),
            interrupt: 0x00,
        }
    }

    pub fn step(&mut self, cycles: usize) -> Result<(), String> {
        self.gpu.step(&mut self.irq, cycles)?;
        self.timer.step(&mut self.irq, cycles)?;

        Ok(())
    }

    pub fn reset(&mut self) {
        self.gpu.reset();
        self.timer.reset();
        self.irq.reset();
    }

    pub fn cart_details(&self) -> CartridgeDetails {
        let cart = self.cart.as_ref().expect("Cartridge is empty");

        cart.details(&self)
    }

    pub fn request_frame(&self) -> &Frame {
        &self.gpu.frame
    }

    pub fn write_u8(&mut self, addr: u16, byte: u8) {
        let cart = self.cart.as_mut().expect("Cartridge is empty");

        match memory_map::map_address(addr) {
            Address::Ram(a) |
            Address::RamShadow(a) => self.ram.write_u8(a, byte),
            Address::Gfx(a) => self.gpu.ram.write_u8(a, byte),
            Address::CartRam(a) => cart.ram.write_u8(a, byte),
            Address::CartRom(a) => cart.rom.write_u8(a, byte),
            Address::ZRam(a) => {
                // if a != 0x00 { <----- This stops the Tetris infinite loops on the Copyright screen (until I can fix it later)
                self.zram.write_u8(a, byte);
                // }
            }
            Address::Oam(a) => self.gpu.sprite_data.write_u8(a, byte),
            Address::Unused(_) => (),
            Address::Io(a) => {
                match a {
                    0x00 => println!("err: write to joypad not supported"),
                    0x01...0x02 => println!("err: write to serial driver not supported"),
                    0x04...0x07 => self.timer.write_u8(a, byte),
                    0x0F => self.irq.request_flag = byte,
                    0x10...0x26 => println!("err: write to sound driver not supported"),
                    0x40...0x45 => self.gpu.write_u8(a, byte),
                    0x47...0x49 => self.gpu.write_u8(a, byte),
                    0x4A...0x4B => self.gpu.write_u8(a, byte),
                    0x7F => self.mmap_io.write_u8(a, byte),
                    0xFF => self.irq.enable_flag = byte,
                    _ => panic!("write memory mapped I/O in unsupported range: {:04X}", a),
                }
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
            Address::Gfx(value) => self.gpu.ram.read_u8(value),
            Address::CartRam(a) => cart.ram.read_u8(a),
            Address::ZRam(a) => self.zram.read_u8(a),
            Address::Oam(a) => self.gpu.sprite_data.read_u8(a),
            Address::Unused(_) => 0xFF, // Always return high
            Address::Io(a) => {
                match a {
                    0x00 => {
                        println!("err: read from joypad not supported");
                        0
                    }
                    0x01...0x02 => {
                        println!("err: read from serial driver not supported");
                        0
                    }
                    0x04...0x07 => self.timer.read_u8(a), 
                    0x0F => self.irq.request_flag,
                    0x10...0x26 => {
                        println!("err: read from sound driver not supported");
                        0
                    }
                    0x40...0x45 => self.gpu.read_u8(a), 
                    0x47...0x49 => self.gpu.read_u8(a),
                    0x4A...0x4B => self.gpu.read_u8(a),
                    0x7F => self.mmap_io.read_u8(a),
                    0xFF => self.irq.enable_flag,
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
            Address::ZRam(a) => self.zram.read_bytes(r),
            Address::Io(a) => self.mmap_io.read_bytes(r),
            _ => panic!("Unable to read address range: {:?}", r),
        }
    }

    pub fn write_u16(&mut self, addr: u16, val: u16) {
        let cart = self.cart.as_ref().expect("Cartridge is empty");

        match memory_map::map_address(addr) {
            Address::Ram(a) |
            Address::RamShadow(a) => self.ram.write_u16(a, val),
            Address::ZRam(a) => self.zram.write_u16(a, val),
            _ => panic!("Unable to write address: {:#X}", addr),
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
