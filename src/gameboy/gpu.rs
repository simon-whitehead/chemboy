use gameboy::{Interconnect, Memory};

const VRAM_SIZE: usize = 0x4000;

pub struct Gpu {
    pub enabled: bool,
    pub ram: Memory,
    control_register: u8,
    ly: u8,

    cycles: isize,
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            enabled: true,
            ram: Memory::new(VRAM_SIZE),
            control_register: 0,
            ly: 0,
            cycles: 0,
        }
    }

    pub fn step(&mut self, cycles: usize) {
        let cycles = cycles as isize;

        if !self.enabled {
            return;
        }

        self.cycles -= cycles;
        if self.cycles < 0 {
            self.cycles = 0x1C8; // it takes 456 CPU clock cycles to draw 1 LCD scanline
            self.ly = (self.ly + 0x01) % 0x9A; // LY can only be within 0...153
            if self.ly >= 0x90 {
                // V-Blank
                // println!("######## V-BLANK");
            }
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0x40 => self.control_register,
            0x44 => self.ly,
            _ => panic!("tried to read GPU memory that is not mapped"),
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            0x40 => self.control_register = val,
            0x44 => self.ly = val,
            _ => panic!("tried to write GPU memory that is not mapped"),
        }
    }
}
