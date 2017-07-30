use std::ops::Range;

use gameboy::Memory;

const VRAM_SIZE: usize = 0x4000;

pub struct Gfx {
    pub ram: Memory,
}

impl Gfx {
    pub fn new() -> Gfx {
        Gfx { ram: Memory::new(VRAM_SIZE) }
    }
}