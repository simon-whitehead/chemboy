
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

#[cfg(test)]
pub mod tests {
    #[test]
    pub fn can_write_to_gfx_memory() {
        let ldhlda = vec![0x32, 0xCB, 0x7C];
        let mut cpu = ::gameboy::Cpu::new(true, ldhlda);

        cpu.step();

        // Graphics RAM
    }
}
