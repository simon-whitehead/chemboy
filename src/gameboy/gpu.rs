use std::ops::Range;

use gameboy::Memory;

const VRAM_SIZE: usize = 0x4000;

pub struct Gpu {
    pub ram: Memory,
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu { ram: Memory::new(VRAM_SIZE) }
    }
}