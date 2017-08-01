use gameboy::{Interconnect, Memory};

const VRAM_SIZE: usize = 0x4000;

pub struct Gpu {
    pub enabled: bool,
    pub ram: Memory,
    ly: u8,

    cycles: isize,
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            enabled: true,
            ram: Memory::new(VRAM_SIZE),
            ly: 0,
            cycles: 0,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn step(&mut self, cycles: u8) {
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
            0x44 => self.ly,
            _ => panic!("tried to read GPU memory that is not mapped"),
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            0x44 => self.write_lcdc_y(val),
            _ => panic!("tried to write GPU memory that is not mapped"),
        }
    }

    pub fn write_lcdc_y(&mut self, val: u8) {
        self.ly = val;
    }
}