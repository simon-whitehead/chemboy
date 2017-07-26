
pub struct Gfx {
    pub ram: [u8; 8192],
}

impl Gfx {
    pub fn new() -> Gfx {
        Gfx { ram: [255u8; 8192] }
    }

    pub fn write_u8(&mut self, addr: u16, byte: u8) {
        self.ram[addr as usize] = byte;
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        let h = self.ram[(addr - 1) as usize];
        let l = self.ram[addr as usize];

        let result: u16 = ((h as u16) << 8) | l as u16;

        result
    }
}