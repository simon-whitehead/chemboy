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

        cpu.step(&interconnect);

        assert_eq!(0x3A, cpu.registers.h);
        assert_eq!(0x5B, cpu.registers.l);
    }

    #[test]
    fn ld_c_imm8() {
        let (mut cpu, interconnect) = create_cpu(gb_asm![0x0E 0xA4]);

        cpu.step(&interconnect);

        assert_eq!(0xA4, cpu.registers.c);
    }

    fn create_cpu(rom: Vec<u8>) -> (Cpu, Interconnect) {
        (Cpu::new(false, rom), Interconnect::new())
    }
}