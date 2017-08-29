use std::ops::Range;

use gameboy::{Gpu, Irq, Memory, Timer};
use gameboy::cartridge::{Cartridge, CartridgeDetails};
use gameboy::frame::Frame;
use gameboy::irq::Interrupt;
use gameboy::joypad::{Joypad, JoypadButton};
use super::memory_map::{self, Address};

const MAIN_MEM_SIZE: usize = 0x2000;
const ZRAM_SIZE: usize = 0x80;
const MMAP_SIZE: usize = 0x80;

const BOOT_ROM: &'static [u8] = include_bytes!("boot_rom.gb");

pub struct Interconnect {
    pub booting: bool,
    pub boot_rom: Memory,
    pub gpu: Gpu,
    pub ram: Memory,
    pub zram: Memory,
    pub mmap_io: Memory,
    pub unused_memory: Memory,
    pub timer: Timer,
    pub irq: Irq,
    pub cart: Option<Cartridge>,
    pub interrupt: u8,
    pub joypad: Joypad,
}

impl Interconnect {
    pub fn new() -> Interconnect {
        Interconnect {
            booting: true,
            boot_rom: Self::init_boot_rom(true),
            gpu: Gpu::new(),
            ram: Memory::new(MAIN_MEM_SIZE),
            zram: Memory::new(ZRAM_SIZE),
            mmap_io: Memory::new(MMAP_SIZE),
            unused_memory: Memory::new(0x60),
            timer: Timer::new(),
            irq: Irq::new(),
            cart: None,
            interrupt: 0x00,
            joypad: Joypad::new(),
        }
    }

    pub fn with_cart(cart: Cartridge, boot_rom_enabled: bool) -> Interconnect {
        Interconnect {
            booting: true,
            boot_rom: Self::init_boot_rom(boot_rom_enabled),
            gpu: Gpu::new(),
            ram: Memory::new(MAIN_MEM_SIZE),
            zram: Memory::new(ZRAM_SIZE),
            mmap_io: Memory::new(MMAP_SIZE),
            unused_memory: Memory::new(0x60),
            timer: Timer::new(),
            irq: Irq::new(),
            cart: Some(cart),
            interrupt: 0x00,
            joypad: Joypad::new(),
        }
    }

    fn init_boot_rom(boot_rom_enabled: bool) -> Memory {
        if boot_rom_enabled {
            let mut mem = Memory::new(BOOT_ROM.len());
            mem.write_bytes(0x00, &BOOT_ROM);
            mem
        } else {
            // This is:
            //
            // LD A, $01
            // LD ($FF00+$32), A
            //

            let mut mem = Memory::new(0x04);
            mem.write_bytes(0x00, &[0x3E, 0x01, 0xE0, 0x50]);
            mem
        }
    }

    pub fn step(&mut self, cycles: usize) -> Result<(), String> {
        self.gpu.step(&mut self.irq, cycles)?;
        self.timer.step(&mut self.irq, cycles)?;
        self.joypad.step(&mut self.irq, cycles)?;

        Ok(())
    }

    pub fn reset(&mut self) {
        self.booting = true;
        self.gpu.reset();
        self.timer.reset();
        self.irq.reset();
    }

    pub fn cart_details(&self) -> &CartridgeDetails {
        let cart = self.cart.as_ref().expect("Cartridge is empty");

        &cart.details
    }

    pub fn request_frame(&self) -> &Frame {
        &self.gpu.frame
    }

    pub fn press(&mut self, button: JoypadButton) {
        self.joypad.press(button);
    }

    pub fn unpress(&mut self, button: JoypadButton) {
        self.joypad.unpress(button);
    }

    pub fn write_u8(&mut self, addr: u16, byte: u8) {
        // Special case - DMA transfer
        if addr == 0xFF46 {
            self.dma_transfer(byte);
            return;
        }
        let cart = self.cart.as_mut().expect("Cartridge is empty");

        match memory_map::map_address(addr) {
            Address::Ram(a) => self.ram.write_u8(a, byte),
            Address::RamShadow(a) => self.ram.write_u8(a, byte),
            Address::Gfx(a) => self.gpu.ram.write_u8(a, byte),
            Address::CartRam(a) => cart.write_ram_u8(a, byte),
            Address::CartRom(a) => cart.write_rom_u8(a, byte),
            Address::ZRam(a) => {
                // if a != 0x00 {
                self.zram.write_u8(a, byte);
                // }
            }
            Address::Oam(a) => self.gpu.sprite_data.write_u8(a, byte),
            Address::Unused(_) => (),
            Address::Io(a) => {
                match a {
                    0x00 => self.joypad.from_u8(byte, &mut self.irq),
                    0x01...0x02 => (), // println!("err: write to serial driver not supported"),
                    0x04...0x07 => self.timer.write_u8(a, byte),
                    0x0F => self.irq.request_flag = byte,
                    0x10...0x26 => (), // println!("err: write to sound driver not supported"),
                    0x30...0x3F => (), // println!("err: write to wave pattern RAM not supported"),
                    0x40...0x45 => self.gpu.write_u8(a, byte),
                    0x47...0x49 => self.gpu.write_u8(a, byte),
                    0x50 => {
                        if self.booting {
                            self.booting = false;
                            self.irq.request(Interrupt::LoadGame);
                        } else {
                            self.mmap_io.write_u8(a, byte);
                        }
                    }
                    0x4A...0x4B => self.gpu.write_u8(a, byte),
                    0x7F => self.mmap_io.write_u8(a, byte),
                    _ => panic!("write memory mapped I/O in unsupported range: {:04X}", a),
                }
            }
            Address::InterruptEnableRegister(a) => self.irq.enable_flag = byte,
            _ => {
                panic!("Unable to write byte to: {:#X}, invalid memory region.",
                       addr)
            }
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        let cart = self.cart.as_ref().expect("Cartridge is empty");

        match memory_map::map_address(addr) {
            Address::Ram(addr) => self.ram.read_u8(addr),
            Address::RamShadow(addr) => self.ram.read_u8(addr),
            Address::CartRom(addr) => {
                if self.booting {
                    self.boot_rom.read_u8(addr)
                } else {
                    cart.read_rom_u8(addr)
                }
            }
            Address::CartRomOtherBank(addr) => cart.read_rom_u8(addr),
            Address::Gfx(value) => self.gpu.ram.read_u8(value),
            Address::CartRam(a) => cart.read_ram_u8(a),
            Address::ZRam(a) => self.zram.read_u8(a),
            Address::Oam(a) => self.gpu.sprite_data.read_u8(a),
            Address::Unused(_) => 0xFF, // Always return high
            Address::Io(a) => {
                match a {
                    0x00 => self.joypad.data,
                    0x01...0x02 => {
                        // println!("err: read from serial driver not supported");
                        0
                    }
                    0x04...0x07 => self.timer.read_u8(a), 
                    0x0F => self.irq.request_flag,
                    0x10...0x26 => {
                        // println!("err: read from sound driver not supported");
                        0
                    }
                    0x30...0x3F => {
                        // println!("err: write to wave pattern RAM not supported");
                        0
                    }
                    0x40...0x45 => self.gpu.read_u8(a), 
                    0x47...0x49 => self.gpu.read_u8(a),
                    0x4A...0x4B => self.gpu.read_u8(a),
                    0x7F => self.mmap_io.read_u8(a),
                    n @ _ => panic!("read memory mapped I/O in unsupported range: {:04X}", n),
                }
            }
            Address::InterruptEnableRegister(a) => self.irq.enable_flag,
            _ => panic!("Unable to read address: {:#X}", addr),
        }
    }

    pub fn read_bytes(&self, r: Range<u16>) -> &[u8] {
        panic!("CALLED, APPARENTLY");
        let cart = self.cart.as_ref().expect("Cartridge is empty");

        match memory_map::map_address(r.start) {
            Address::Ram(_) | Address::Gfx(_) => self.gpu.ram.read_bytes(r),
            Address::ZRam(a) => self.zram.read_bytes(r),
            Address::Io(a) => self.mmap_io.read_bytes(r),
            _ => panic!("Unable to read address range: {:?}", r),
        }
    }

    pub fn write_u16(&mut self, addr: u16, val: u16) {
        let cart = self.cart.as_mut().expect("Cartridge is empty");

        match memory_map::map_address(addr) {
            Address::CartRam(a) => cart.write_ram_u16(a, val),
            Address::Gfx(a) => self.gpu.write_u16(a, val),
            Address::Io(a) => self.mmap_io.write_u16(a, val),
            Address::Oam(a) => self.gpu.sprite_data.write_u16(a, val),
            Address::Ram(a) => self.ram.write_u16(a, val),
            Address::RamShadow(a) => self.ram.write_u16(a, val),
            Address::Unused(a) => self.unused_memory.write_u16(a, val),
            Address::ZRam(a) => self.zram.write_u16(a, val),
            _ => panic!("Unable to write address: {:#X}", addr),
        }
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        let cart = self.cart.as_ref().expect("Cartridge is empty");

        match memory_map::map_address(addr) {
            Address::Ram(addr) => self.ram.read_u16(addr),
            Address::RamShadow(addr) => self.ram.read_u16(addr),
            Address::CartRom(addr) => {
                if self.booting {
                    self.boot_rom.read_u16(addr)
                } else {
                    cart.read_rom_u16(addr)
                }
            }
            Address::CartRomOtherBank(addr) => cart.read_rom_u16(addr),
            Address::Gfx(value) => self.gpu.ram.read_u16(value),
            Address::CartRam(a) => cart.read_ram_u16(a),
            Address::ZRam(a) => self.zram.read_u16(a),
            Address::Io(a) => self.mmap_io.read_u16(a),
            _ => panic!("Unable to read address: {:#X}", addr),
        }
    }

    pub fn write_bytes(&mut self, addr: u16, bytes: &[u8]) {
        let cart = self.cart.as_mut().expect("Cartridge is empty");
        match memory_map::map_address(addr) {
            // Address::CartRom(addr) => cart.write_rom_bytes(addr, bytes),
            _ => panic!("write_bytes not mapped for specified memory region"),
        }
    }

    fn dma_transfer(&mut self, byte: u8) {
        let addr = (byte as u16) << 0x08; // "The written value specifies the transfer source address divided by 0x100"
        for x in 0..0xA0 {
            let val = self.read_u8(addr + x);
            self.write_u8(0xFE00 + x, val);
        }
    }
}
