
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

    pub fn read_word(&self, addr: u16) -> u16 {
        let addr = addr - 1;
        let h = self.ram[addr as usize];
        let l = self.ram[(addr + 1) as usize];

        let result: u16 = ((h as u16) << 8) | l as u16;

        result
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use ::gameboy::opcode::*;

    #[test]
    pub fn can_write_to_gfx_memory() {
        let ldhlda = gb_asm![
            XOR_A
            LD_HL_u16 0xFF 0x9F
            LD_HLD_A
        ];

        let mut cpu = ::gameboy::Cpu::new(true, ldhlda);

        cpu.step();
        cpu.step();
        cpu.step();

        let val = cpu.interconnect.read_word(cpu.registers.get_hl() + 1);
        assert_eq!(0, val);
    }
}
