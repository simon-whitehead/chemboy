#[macro_use]
extern crate gbrs;

#[cfg(test)]
mod tests {

    use gbrs::gameboy::{Cpu, Interconnect};

    #[test]
    fn xor_a_xors_a() {
        let (mut cpu, interconnect) = create_cpu(gb_asm![0xAF]);

        cpu.registers.a = 0xFF;
        cpu.step(&interconnect);

        assert_eq!(0, cpu.registers.a);
    }

    #[test]
    fn ld_hl_imm16() {
        let (mut cpu, interconnect) = create_cpu(gb_asm![0x21 0x5B 0x3A]);

        cpu.registers.a = 0xFF;
        cpu.step(&interconnect);

        assert_eq!(0x3A, cpu.registers.h);
        assert_eq!(0x5B, cpu.registers.l);
    }

    fn create_cpu(rom: Vec<u8>) -> (Cpu, Interconnect) {
        (Cpu::new(false, rom), Interconnect::new())
    }
}