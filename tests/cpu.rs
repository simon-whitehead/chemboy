#[macro_use]
extern crate gbrs;

#[cfg(test)]
mod tests {

    use gbrs::gameboy::{Cartridge, Cpu, Interconnect};

    #[test]
    fn add_a_a() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x87]);

        cpu.registers.a = 0xAA;
        cpu.step(&mut interconnect);

        assert_eq!(0x54, cpu.registers.a);
        assert_eq!(false, cpu.registers.flags.zero);
        assert_eq!(true, cpu.registers.flags.half_carry);
        assert_eq!(false, cpu.registers.flags.negative);
        assert_eq!(true, cpu.registers.flags.carry);
    }
    #[test]
    fn add_hl_de() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x19]);

        cpu.registers.set_hl(0x8A23);
        cpu.registers.set_de(0x0605);
        cpu.step(&mut interconnect);

        assert_eq!(0x9028, cpu.registers.get_hl());
        assert_eq!(true, cpu.registers.flags.half_carry);
        assert_eq!(false, cpu.registers.flags.negative);
        assert_eq!(false, cpu.registers.flags.carry);
    }

    #[test]
    fn and_a() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xA7]);

        cpu.registers.a = 0x5A;
        cpu.step(&mut interconnect);

        assert_eq!(0x5A, cpu.registers.a);
        assert_eq!(false, cpu.registers.flags.zero);
        assert_eq!(true, cpu.registers.flags.half_carry);
        assert_eq!(false, cpu.registers.flags.negative);
        assert_eq!(false, cpu.registers.flags.carry);
    }

    #[test]
    fn and_c() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xA1]);

        cpu.registers.a = 0x5A;
        cpu.registers.c = 0x38;
        cpu.step(&mut interconnect);

        assert_eq!(0x18, cpu.registers.a);
        assert_eq!(false, cpu.registers.flags.zero);
        assert_eq!(true, cpu.registers.flags.half_carry);
        assert_eq!(false, cpu.registers.flags.negative);
        assert_eq!(false, cpu.registers.flags.carry);
    }

    #[test]
    fn and_imm8() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xE6 0x38]);

        cpu.registers.a = 0x5A;
        cpu.step(&mut interconnect);

        assert_eq!(0x18, cpu.registers.a);
        assert_eq!(false, cpu.registers.flags.zero);
        assert_eq!(true, cpu.registers.flags.half_carry);
        assert_eq!(false, cpu.registers.flags.negative);
        assert_eq!(false, cpu.registers.flags.carry);
    }

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
    fn cpl() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x2F]);

        cpu.registers.a = 0x35;
        cpu.step(&mut interconnect);

        assert_eq!(0xCA, cpu.registers.a);
        assert_eq!(true, cpu.registers.flags.negative);
        assert_eq!(true, cpu.registers.flags.half_carry);
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
    fn dec_a() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x3D]);

        cpu.registers.a = 0x01;
        cpu.step(&mut interconnect);

        assert_eq!(0x00, cpu.registers.a);
        assert_eq!(true, cpu.registers.flags.zero);
        assert_eq!(false, cpu.registers.flags.half_carry);
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
    fn ei() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xFB]);

        cpu.registers.flags.ime = false;
        cpu.step(&mut interconnect);

        assert_eq!(true, cpu.registers.flags.ime);
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
    fn inc_de() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x13]);

        cpu.registers.set_de(0xCFFE);
        cpu.step(&mut interconnect);

        assert_eq!(0xCFFF, cpu.registers.get_de());
    }

    #[test]
    fn inc_e() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x1C]);

        cpu.registers.e = 0xFF;
        cpu.step(&mut interconnect);

        assert_eq!(0x00, cpu.registers.e);
        assert_eq!(true, cpu.registers.flags.zero);
        assert_eq!(true, cpu.registers.flags.half_carry);
        assert_eq!(false, cpu.registers.flags.negative);
    }

    #[test]
    fn inc_hl() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x23]);

        cpu.registers.set_hl(0xCFFE);
        cpu.step(&mut interconnect);

        assert_eq!(0xCFFF, cpu.registers.get_hl());
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
    fn jr_z_imm8() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x28 0x01 0x0C 0x00]);

        cpu.registers.flags.zero = false;
        cpu.registers.c = 0xFE;
        cpu.step(&mut interconnect);

        assert_eq!(0xFE, cpu.registers.c);
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
    fn ld_a_c() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x79]);

        cpu.registers.c = 0xE5;
        cpu.registers.a = 0xFF;
        cpu.step(&mut interconnect);

        assert_eq!(0xE5, cpu.registers.a);
    }

    #[test]
    fn ld_a_de() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x1A]);

        interconnect.write_u8(0xC00A, 0xBA);
        cpu.registers.set_de(0xC00A);
        cpu.step(&mut interconnect);

        assert_eq!(0xBA, cpu.registers.a);
    }

    #[test]
    fn ld_a_h() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x7C]);

        cpu.registers.h = 0xE5;
        cpu.registers.a = 0xFF;
        cpu.step(&mut interconnect);

        assert_eq!(0xE5, cpu.registers.a);
    }

    #[test]
    fn ld_a_ff00_imm8() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xF0 0x44]);

        interconnect.write_u8(0xFF44, 0xA9);
        cpu.step(&mut interconnect);

        assert_eq!(0xA9, cpu.registers.a);
    }

    #[test]
    fn ld_a_hl() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x7E]);

        cpu.registers.a = 0xE5;
        interconnect.write_u8(0xC000, 0xC3);
        cpu.registers.set_hl(0xC000);
        cpu.step(&mut interconnect);

        assert_eq!(0xC3, cpu.registers.a);
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
    fn ld_a_imm16() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xFA 0x0A 0xC0]);

        cpu.registers.a = 0xFF;
        interconnect.write_u8(0xC00A, 0xCC);
        cpu.step(&mut interconnect);

        assert_eq!(0xCC, cpu.registers.a);
    }

    #[test]
    fn ld_b_a() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x47]);

        cpu.registers.a = 0x61;
        cpu.registers.b = 0x00;
        cpu.step(&mut interconnect);

        assert_eq!(0x61, cpu.registers.b);
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
    fn ld_c_a() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x4F]);

        cpu.registers.a = 0x61;
        cpu.registers.c = 0x00;
        cpu.step(&mut interconnect);

        assert_eq!(0x61, cpu.registers.c);
    }

    #[test]
    fn ld_c_imm8() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x0E 0xA4]);

        cpu.step(&mut interconnect);

        assert_eq!(0xA4, cpu.registers.c);
    }

    #[test]
    fn ld_d_h() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x54]);

        cpu.registers.h = 0xCF;
        cpu.step(&mut interconnect);

        assert_eq!(0xCF, cpu.registers.d);
    }

    #[test]
    fn ld_d_hl() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x56]);

        cpu.registers.set_hl(0xC000);
        interconnect.write_u8(0xC000, 0xBC);
        cpu.step(&mut interconnect);

        assert_eq!(0xBC, cpu.registers.d);
    }

    #[test]
    fn ld_d_imm8() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x16 0xA4]);

        cpu.registers.d = 0x00;
        cpu.step(&mut interconnect);

        assert_eq!(0xA4, cpu.registers.d);
    }

    #[test]
    fn ld_de_a() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x12]);

        cpu.registers.a = 0xEA;
        cpu.registers.set_de(0xC002);
        cpu.step(&mut interconnect);

        assert_eq!(0xEA, interconnect.read_u8(cpu.registers.get_de()));
    }

    #[test]
    fn ld_de_imm16() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x11 0xEF 0xCD]);

        cpu.step(&mut interconnect);

        assert_eq!(0xCDEF, cpu.registers.get_de());
    }

    #[test]
    fn ld_e_a() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x5F]);

        cpu.registers.a = 0xCD;
        cpu.registers.e = 0x00;
        cpu.step(&mut interconnect);

        assert_eq!(0xCD, cpu.registers.e);
    }

    #[test]
    fn ld_e_hl() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x5E]);

        cpu.registers.set_hl(0xC000);
        interconnect.write_u8(0xC000, 0xBC);
        cpu.step(&mut interconnect);

        assert_eq!(0xBC, cpu.registers.e);
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
    fn ld_hli_a() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x22]);

        cpu.registers.a = 0xE5;
        cpu.registers.set_hl(0xC000);
        interconnect.write_u8(0xC000, 0xE3);
        cpu.step(&mut interconnect);

        assert_eq!(0xE5, interconnect.read_u8(0xC000));
        assert_eq!(0xC001, cpu.registers.get_hl());
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
    fn ld_l_b() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x68]);

        cpu.registers.b = 0xCC;
        cpu.step(&mut interconnect);

        assert_eq!(0xCC, cpu.registers.l);
    }

    #[test]
    fn ld_sp_imm16() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0x31 0xCF 0xF7]);

        cpu.step(&mut interconnect);

        assert_eq!(0xF7CF, cpu.registers.sp);
    }

    #[test]
    fn or_b() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xB0]);

        cpu.registers.a = 0x36;
        cpu.registers.b = 0x77;
        cpu.step(&mut interconnect);

        assert_eq!(0x77, cpu.registers.a);
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
    fn pop_af() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xF1]);

        interconnect.write_u16(0xFFFC, 0xCF91);
        cpu.registers.sp = 0xFFFC;
        cpu.step(&mut interconnect);

        assert_eq!(0xCF, cpu.registers.a);
        assert_eq!(0x91, cpu.registers.f);
        assert_eq!(0xFFFE, cpu.registers.sp);
    }

    #[test]
    fn pop_bc() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xC1]);

        interconnect.write_u16(0xFFFC, 0xCF91);
        cpu.registers.sp = 0xFFFC;
        cpu.step(&mut interconnect);

        assert_eq!(0xCF, cpu.registers.b);
        assert_eq!(0x91, cpu.registers.c);
        assert_eq!(0xFFFE, cpu.registers.sp);
    }

    #[test]
    fn pop_de() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xD1]);

        interconnect.write_u16(0xFFFC, 0xCF91);
        cpu.registers.sp = 0xFFFC;
        cpu.step(&mut interconnect);

        assert_eq!(0xCF, cpu.registers.d);
        assert_eq!(0x91, cpu.registers.e);
        assert_eq!(0xFFFE, cpu.registers.sp);
    }

    #[test]
    fn pop_hl() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xE1]);

        interconnect.write_u16(0xFFFC, 0xCF91);
        cpu.registers.sp = 0xFFFC;
        cpu.step(&mut interconnect);

        assert_eq!(0xCF, cpu.registers.h);
        assert_eq!(0x91, cpu.registers.l);
        assert_eq!(0xFFFE, cpu.registers.sp);
    }

    #[test]
    fn push_af() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xF5]);

        cpu.registers.set_af(0xCFA5);
        cpu.step(&mut interconnect);

        assert_eq!(0xCFA5, interconnect.read_u16(cpu.registers.sp as u16));
    }

    #[test]
    fn push_bc() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xC5]);

        cpu.registers.set_bc(0xCFA5);
        cpu.step(&mut interconnect);

        assert_eq!(0xCFA5, interconnect.read_u16(cpu.registers.sp as u16));
    }

    #[test]
    fn push_de() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xD5]);

        cpu.registers.set_de(0xCFA5);
        cpu.step(&mut interconnect);

        assert_eq!(0xCFA5, interconnect.read_u16(cpu.registers.sp as u16));
    }

    #[test]
    fn push_hl() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xE5]);

        cpu.registers.set_hl(0xCFA5);
        cpu.step(&mut interconnect);

        assert_eq!(0xCFA5, interconnect.read_u16(cpu.registers.sp as u16));
    }

    #[test]
    fn res_0_a() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xCB 0x87]);

        cpu.registers.a = 0x3B;
        cpu.step(&mut interconnect);

        assert_eq!(0x3A, cpu.registers.a);
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
        cpu.step(&mut interconnect); // step over 'INC C'
        assert_eq!(0x3E, cpu.registers.c); // C should be 0x3C + 2
    }

    #[test]
    fn ret_nz() {
        let (mut cpu, mut interconnect) =
            create_cpu(gb_asm![0x00 0x00 0xCD 0x0C 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x0C 0xC0]);
        //                               ^^^^^^^^^ jump to the 'INC C' opcode here --------^^^^

        cpu.registers.c = 0x00;
        cpu.step(&mut interconnect); // step over NOP
        cpu.step(&mut interconnect); // step over NOP
        cpu.step(&mut interconnect); // step over CALL 0x000C (jump to byte 12)
        cpu.step(&mut interconnect); // step over 'INC C'
        cpu.step(&mut interconnect); // step over 'RET NZ', jumping back to byte 5 because C will make f.zero == false
        assert_eq!(0x01, cpu.registers.c);
        assert_eq!(0x05, cpu.registers.pc);
    }

    #[test]
    fn ret_z() {
        let (mut cpu, mut interconnect) =
            create_cpu(gb_asm![0x00 0x00 0xCD 0x0C 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x0C 0xC8]);
        //                               ^^^^^^^^^ jump to the 'INC C' opcode here --------^^^^

        cpu.registers.c = 0xFF;
        cpu.step(&mut interconnect); // step over NOP
        cpu.step(&mut interconnect); // step over NOP
        cpu.step(&mut interconnect); // step over CALL 0x000C (jump to byte 12)
        cpu.step(&mut interconnect); // step over 'INC C'
        cpu.step(&mut interconnect); // step over 'RET NZ', jumping back to byte 5 because C will make f.zero == true
        assert_eq!(0x00, cpu.registers.c);
        assert_eq!(0x05, cpu.registers.pc);
    }

    #[test]
    fn swap_a() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xCB 0x37]);

        cpu.registers.a = 0xC3;
        cpu.step(&mut interconnect);

        assert_eq!(0x3C, cpu.registers.a);
    }

    #[test]
    fn xor_a() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xAF]);

        cpu.registers.a = 0xFF;
        cpu.step(&mut interconnect);

        assert_eq!(0, cpu.registers.a);
    }

    #[test]
    fn xor_b() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xA8]);

        cpu.registers.a = 0xFF;
        cpu.registers.b = 0xFF;
        cpu.step(&mut interconnect);

        assert_eq!(0, cpu.registers.a);
    }

    #[test]
    fn xor_c() {
        let (mut cpu, mut interconnect) = create_cpu(gb_asm![0xA9]);

        cpu.registers.a = 0xFF;
        cpu.registers.c = 0xFF;
        cpu.step(&mut interconnect);

        assert_eq!(0, cpu.registers.a);
    }

    fn create_cpu(rom: Vec<u8>) -> (Cpu, Interconnect) {
        let cart = Cartridge::with_rom(rom);
        (Cpu::new(false), Interconnect::with_cart(cart))
    }
}
