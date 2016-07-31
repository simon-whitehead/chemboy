
pub struct Gfx {
    pub ram: [u8; 8192],
}

impl Gfx {
    pub fn new() -> Gfx {
        Gfx { ram: [0u8; 8192] }
    }

    pub fn write_byte(&mut self, addr: u16, byte: u8) {
        self.ram[addr as usize] = byte;
    }
}
