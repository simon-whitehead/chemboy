
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
        let h = self.ram[addr as usize];
        let l = self.ram[(addr + 1) as usize];

        let result: u16 = ((h as u16) << 8) | l as u16;

        result
    }
}

#[cfg(test)]
pub mod tests {
    #[test]
    pub fn can_write_to_gfx_memory() {
        let ldhlda = vec![0x21, 0xFF, 0x9F, 0x32];
        let mut cpu = ::gameboy::Cpu::new(true, ldhlda);

        cpu.step();

        // TODO: Test that graphics memory is supported for the LD (HL-),A opcode (0x32)
        match ::gameboy::memory_map::map_address(cpu.registers.get_hl() + 1) {
            ::gameboy::memory_map::Address::Gfx(value) => {
                let val = cpu.interconnect.read_word(value);
                assert_eq!(0, val);
            }
            _ => panic!("Invalid memory address"),
        }
    }
}
