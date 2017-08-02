#[macro_use]
extern crate gbrs;

#[cfg(test)]
mod tests {

    use gbrs::gameboy::{Cartridge, Cpu, Interconnect};

    #[test]
    fn call() {
        let (mut cpu, mut interconnect) =
            create_cpu(gb_asm![0x00 0x00 0xCD 0x0C 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x0C]);
        //                               ^^^^^^^^^ jump to the 'INC C' opcode here --------^^^^

        cpu.registers.c = 0x3C;
        cpu.step(&mut interconnect); // step over NOP
        cpu.step(&mut interconnect); // step over NOP
        cpu.step(&mut interconnect); // step over CALL 0x000C (jump to byte 12)
        assert_eq!(0x0C, cpu.registers.pc); // program counter should be at byte 12
        assert_eq!(0x05, interconnect.read_u16(cpu.registers.sp as u16)); // return address on stack should be byte 5 (2 nops + 3 bytes for the call)
        cpu.step(&mut interconnect); // step over 'INC C'
        assert_eq!(0x3D, cpu.registers.c); // C should be 0x3C + 1
    }

    #[test]
    fn cp_n() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xFE 0x3C]);

        cpu.registers.a = 0x3C;
        cpu.step(&mut interconnect);

        assert_eq!(true, cpu.registers.flags.zero);
        assert_eq!(false, cpu.registers.flags.half_carry);
        assert_eq!(true, cpu.registers.flags.negative);
        assert_eq!(false, cpu.registers.flags.carry);
    }

    #[test]
    fn dec_b() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x05]);

        cpu.registers.b = 0x01;
        cpu.step(&mut interconnect);

        assert_eq!(0x00, cpu.registers.b);
        assert_eq!(true, cpu.registers.flags.zero);
        assert_eq!(false, cpu.registers.flags.half_carry);
    }

    #[test]
    fn dec_b_2() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x05]);

        cpu.registers.b = 0x00;
        cpu.step(&mut interconnect);

        assert_eq!(0xFF, cpu.registers.b);
        assert_eq!(false, cpu.registers.flags.zero);
        assert_eq!(true, cpu.registers.flags.half_carry);
    }

    #[test]
    fn dec_bc() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x0B]);

        cpu.registers.set_bc(0x33);
        cpu.step(&mut interconnect);

        assert_eq!(0x32, cpu.registers.get_bc());
    }

    #[test]
    fn dec_c() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x0D]);

        cpu.registers.c = 0x01;
        cpu.step(&mut interconnect);

        assert_eq!(0x00, cpu.registers.c);
        assert_eq!(true, cpu.registers.flags.zero);
        assert_eq!(false, cpu.registers.flags.half_carry);
    }

    #[test]
    fn dec_c_2() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x0D]);

        cpu.registers.c = 0x00;
        cpu.step(&mut interconnect);

        assert_eq!(0xFF, cpu.registers.c);
        assert_eq!(false, cpu.registers.flags.zero);
        assert_eq!(true, cpu.registers.flags.half_carry);
    }

    #[test]
    fn di() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xF3]);

        cpu.registers.flags.ime = true;
        cpu.step(&mut interconnect);

        assert_eq!(false, cpu.registers.flags.ime);
    }

    #[test]
    fn inc_c() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x0C]);

        cpu.registers.c = 0xFF;
        cpu.step(&mut interconnect);

        assert_eq!(0x00, cpu.registers.c);
        assert_eq!(true, cpu.registers.flags.zero);
        assert_eq!(true, cpu.registers.flags.half_carry);
        assert_eq!(false, cpu.registers.flags.negative);
    }

    #[test]
    fn jr_nz_imm8() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xAF 0x20 0xFD]);

        cpu.step(&mut interconnect);
        cpu.registers.flags.zero = false;
        cpu.step(&mut interconnect);

        assert_eq!(0, cpu.registers.pc);

        cpu.registers.a = 0xFF;
        cpu.step(&mut interconnect);

        assert_eq!(0, cpu.registers.a);
    }

    #[test]
    fn ld_a_b() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x78]);

        cpu.registers.b = 0xE5;
        cpu.registers.a = 0xFF;
        cpu.step(&mut interconnect);

        assert_eq!(0xE5, cpu.registers.a);
    }

    #[test]
    fn ld_a_hli() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x2A]);

        cpu.registers.a = 0xE5;
        interconnect.write_u8(0xC000, 0xC3);
        cpu.registers.set_hl(0xC000);
        cpu.step(&mut interconnect);

        assert_eq!(0xC3, cpu.registers.a);
        assert_eq!(0xC001, cpu.registers.get_hl());
    }

    #[test]
    fn ld_a_imm8() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x3E 0xA3]);

        cpu.registers.a = 0xE5;
        cpu.step(&mut interconnect);

        assert_eq!(0xA3, cpu.registers.a);
    }

    #[test]
    fn ld_b_imm8() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x06 0xA5]);

        cpu.step(&mut interconnect);

        assert_eq!(0xA5, cpu.registers.b);
    }

    #[test]
    fn ld_bc_imm16() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x01 0xCF 0xA1]);

        cpu.step(&mut interconnect);

        assert_eq!(0xA1, cpu.registers.b);
        assert_eq!(0xCF, cpu.registers.c);
    }

    #[test]
    fn ld_c_imm8() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x0E 0xA4]);

        cpu.step(&mut interconnect);

        assert_eq!(0xA4, cpu.registers.c);
    }

    #[test]
    fn ld_a_ff00_imm8() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xF0 0x44]);

        interconnect.write_u8(0xFF44, 0xA9);
        cpu.step(&mut interconnect);

        assert_eq!(0xA9, cpu.registers.a);
    }

    #[test]
    fn ld_ff00_imm8_a() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xE0 0x44]);

        cpu.registers.a = 0xAF;
        cpu.step(&mut interconnect);

        assert_eq!(0xAF, interconnect.read_u8(0xFF44));
    }

    #[test]
    fn ld_ff00_c_a() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xE2]);

        cpu.registers.a = 0xAF;
        cpu.registers.c = 0x47;
        interconnect.write_u8(0xFF47, 0xC3);
        cpu.step(&mut interconnect);

        assert_eq!(0xAF, interconnect.read_u8(0xFF47));
    }

    #[test]
    fn ld_hl_imm8() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x36 0x5B]);

        cpu.registers.set_hl(0x014D);
        cpu.step(&mut interconnect);

        assert_eq!(0x5B, interconnect.read_u8(0x014D));
    }

    #[test]
    fn ld_hl_imm16() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x21 0x5B 0x3A]);

        cpu.step(&mut interconnect);

        assert_eq!(0x3A, cpu.registers.h);
        assert_eq!(0x5B, cpu.registers.l);
    }

    #[test]
    fn ld_hld_a() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x32]);

        cpu.registers.a = 0xE5;
        cpu.registers.set_hl(0xC000);
        interconnect.write_u8(0xC000, 0xE3);
        cpu.step(&mut interconnect);

        assert_eq!(0xE5, interconnect.read_u8(0xC000));
        assert_eq!(0xBFFF, cpu.registers.get_hl());
    }

    #[test]
    fn ld_imm16_a() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xEA 0xC5 0xAF]);

        cpu.registers.a = 0xCC;
        interconnect.write_u8(0xAFC5, 0xE3);
        cpu.step(&mut interconnect);

        assert_eq!(0xCC, interconnect.read_u8(0xAFC5));
    }

    #[test]
    fn ld_sp_imm16() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x31 0xCF 0xF7]);

        cpu.step(&mut interconnect);

        assert_eq!(0xF7CF, cpu.registers.sp);
    }

    #[test]
    fn or_c() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xB1]);

        cpu.registers.a = 0x36;
        cpu.registers.c = 0x77;
        cpu.step(&mut interconnect);

        assert_eq!(0x77, cpu.registers.a);
    }

    #[test]
    fn ret() {
        let (mut cpu, mut interconnect) =
            create_cpu(gb_asm![0x00 0x00 0xCD 0x0C 0x00 0x0C 0x00 0x00 0x00 0x00 0x00 0x00 0x0C 0xC9]);
        //                               ^^^^^^^^^ jump to the 'INC C' opcode here --------^^^^

        cpu.registers.c = 0x3C;
        cpu.step(&mut interconnect); // step over NOP
        cpu.step(&mut interconnect); // step over NOP
        cpu.step(&mut interconnect); // step over CALL 0x000C (jump to byte 12)
        cpu.step(&mut interconnect); // step over 'INC C'
        cpu.step(&mut interconnect); // step over 'RET', jumping back to byte 5
        println!("PC is now: {:04X}", cpu.registers.pc);
        println!("C is now: {:02X}", cpu.registers.c);
        cpu.step(&mut interconnect); // step over 'INC C'
        println!("C is now: {:02X}", cpu.registers.c);
        assert_eq!(0x3E, cpu.registers.c); // C should be 0x3C + 2
    }

    #[test]
    fn xor_a_xors_a() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xAF]);

        cpu.registers.a = 0xFF;
        cpu.step(&mut interconnect);

        assert_eq!(0, cpu.registers.a);
    }

    fn create_cpu(rom: Vec<u8>) -> (Cpu, Interconnect) {
        let cart = Cartridge::with_rom(rom);
        (Cpu::new(false), Interconnect::with_cart(cart))
    }
}
