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

    fn create_cpu(rom: Vec<u8>) -> (Cpu, Interconnect) {
        (Cpu::new(false, rom), Interconnect::new())
    }
}