// Simon Whitehead, 2017

use gameboy::registers;
use gameboy::{Interconnect, Interrupt, MAX_CPU_CYCLES};
use gameboy::opcodes::{ArgumentType, OpCode, Operand};

pub enum CpuSpeed {
    Normal,
    Double,
}

pub struct Cpu {
    pub registers: registers::Registers,
    pub speed: CpuSpeed,
    pub halted: bool,
    pub trace_points: [bool; 15],
}

impl Cpu {
    pub fn new(gameboy_color: bool) -> Cpu {
        Cpu {
            registers: registers::Registers::new(gameboy_color),
            speed: CpuSpeed::Normal,
            halted: false,
            trace_points: [false; 15],
        }
    }

    pub fn reset(&mut self, interconnect: &mut Interconnect) {
        interconnect.reset();
        self.set_initial_values(interconnect);
        self.registers.pc = 0x00;
    }

    pub fn set_initial_values(&mut self, interconnect: &mut Interconnect) {
        self.halted = false;
        self.registers.pc = 0x100;
        self.registers.set_af(0x01B0);
        self.registers.set_bc(0x0013);
        self.registers.set_de(0x00D8);
        self.registers.set_hl(0x014D);
        self.registers.sp = 0xFFFE;

        interconnect.write_u8(0xFF05, 0x00);
        interconnect.write_u8(0xFF06, 0x00);
        interconnect.write_u8(0xFF07, 0x00);
        interconnect.write_u8(0xFF10, 0x80);
        interconnect.write_u8(0xFF11, 0xBF);
        interconnect.write_u8(0xFF12, 0xF3);
        interconnect.write_u8(0xFF14, 0xBF);
        interconnect.write_u8(0xFF16, 0x3F);
        interconnect.write_u8(0xFF17, 0x00);
        interconnect.write_u8(0xFF19, 0xBF);
        interconnect.write_u8(0xFF1A, 0x7F);
        interconnect.write_u8(0xFF1B, 0xFF);
        interconnect.write_u8(0xFF1C, 0x9F);
        interconnect.write_u8(0xFF1E, 0xBF);
        interconnect.write_u8(0xFF20, 0xFF);
        interconnect.write_u8(0xFF21, 0x00);
        interconnect.write_u8(0xFF22, 0x00);
        interconnect.write_u8(0xFF23, 0xBF);
        interconnect.write_u8(0xFF24, 0x77);
        interconnect.write_u8(0xFF25, 0xF3);
        interconnect.write_u8(0xFF26, 0xF1);
        interconnect.write_u8(0xFF40, 0x91);
        interconnect.write_u8(0xFF42, 0x00);
        interconnect.write_u8(0xFF43, 0x00);
        interconnect.write_u8(0xFF45, 0x00);
        interconnect.write_u8(0xFF47, 0xFC);
        interconnect.write_u8(0xFF48, 0xFF);
        interconnect.write_u8(0xFF49, 0xFF);
        interconnect.write_u8(0xFF4A, 0x00);
        interconnect.write_u8(0xFF4B, 0x00);
        interconnect.write_u8(0xFFFF, 0x00);
    }

    fn get_operand_from_opcode(&self, interconnect: &Interconnect, opcode: &OpCode) -> Operand {
        let operand_start = self.registers.pc + 0x01;

        match opcode.argument_type {
            ArgumentType::Implied => Operand::None,
            ArgumentType::Imm8 => Operand::Imm8(interconnect.read_u8(operand_start)),
            ArgumentType::Imm16 => Operand::Imm16(interconnect.read_u16(operand_start)),
            _ => panic!("Unknown opcode argument type"),
        }
    }

    pub fn cycle(&mut self, interconnect: &mut Interconnect) -> Result<(), String> {
        let mut cycles = 0;

        while cycles < self.get_cycles_for_speed() {
            let c = self.step(interconnect)?;
            cycles += c as usize;
            interconnect.step(c as usize)?;
            if self.handle_interrupts(interconnect) > 0x00 {
                self.halted = false;
            }
        }

        Ok(())
    }

    fn get_cycles_for_speed(&self) -> usize {
        match self.speed {
            CpuSpeed::Normal => MAX_CPU_CYCLES,
            CpuSpeed::Double => MAX_CPU_CYCLES << 0x01,
        }
    }

    pub fn handle_interrupts(&mut self, interconnect: &mut Interconnect) -> u8 {
        // Always handle a LoadGame interrupt whether its enabled or not
        if interconnect.irq.requested(&Interrupt::LoadGame) {
            interconnect.irq.unrequest(Interrupt::LoadGame);
            self.set_initial_values(interconnect);
            return 0xFF;
        }

        if !interconnect.irq.enabled {
            return 0x00;
        }

        if interconnect.irq.should_handle(Interrupt::Vblank) {
            interconnect.irq.enabled = false;
            self.call(0x40, interconnect);
            interconnect.irq.unrequest(Interrupt::Vblank);

            return 0x0C;
        }

        if interconnect.irq.should_handle(Interrupt::Lcd) {
            interconnect.irq.enabled = false;
            interconnect.irq.unrequest(Interrupt::Lcd);
            self.call(0x48, interconnect);

            return 0x0C;
        }

        if interconnect.irq.should_handle(Interrupt::Timer) {
            interconnect.irq.enabled = false;
            interconnect.irq.unrequest(Interrupt::Timer);
            self.call(0x50, interconnect);

            return 0x0C;
        }

        if interconnect.irq.should_handle(Interrupt::Serial) {
            interconnect.irq.enabled = false;
            interconnect.irq.unrequest(Interrupt::Serial);
            self.call(0x58, interconnect);

            return 0x0C;
        }

        if interconnect.irq.should_handle(Interrupt::Joypad) {
            interconnect.irq.enabled = false;
            interconnect.irq.unrequest(Interrupt::Joypad);
            self.call(0x60, interconnect);

            return 0x0C;
        }

        0x00
    }

    pub fn step(&mut self, interconnect: &mut Interconnect) -> Result<u8, String> {
        // Do nothing if we're halted
        if self.halted {
            return Ok(0x01);
        }

        let byte = interconnect.read_u8(self.registers.pc);

        if let Some(opcode) = OpCode::from_byte(byte, false) {
            let mut cycles = opcode.cycles;
            let operand = self.get_operand_from_opcode(interconnect, &opcode);

            self.registers.pc += opcode.length;

            match opcode.code {
                0x00 => (),
                0x01 => self.ld_bc_imm16(&operand),
                0x02 => self.ld_bc_a(interconnect),
                0x03 => self.inc_bc(),
                0x04 => self.inc_b(),
                0x05 => self.dec_b(),
                0x06 => self.ld_b_imm8(&operand),
                0x07 => self.rlca(),
                0x08 => self.ld_imm16_ptr_sp(&operand, interconnect),
                0x09 => self.add_hl_bc(),
                0x0A => self.ld_a_bc(interconnect),
                0x0B => self.dec_bc(),
                0x0C => self.inc_c(),
                0x0D => self.dec_c(),
                0x0E => self.ld_c_imm8(&operand),
                0x0F => self.rrca(),
                0x10 => self.halt(), // We're emulating hardware... just HALT instead of STOP
                0x11 => self.ld_de_imm16(&operand),
                0x12 => self.ld_de_a(interconnect),
                0x13 => self.inc_de(),
                0x14 => self.inc_d(),
                0x15 => self.dec_d(),
                0x16 => self.ld_d_imm8(&operand),
                0x17 => self.rla(),
                0x18 => self.jp_imm8(&operand),
                0x19 => self.add_hl_de(),
                0x1A => self.ld_a_de(interconnect),
                0x1B => self.dec_de(),
                0x1C => self.inc_e(),
                0x1D => self.dec_e(),
                0x1E => self.ld_e_imm8(&operand),
                0x1F => self.rra(),
                0x20 => self.jr_nz_imm8(&operand),
                0x21 => self.ld_hl_imm16(&operand),
                0x22 => self.ld_hli_a(interconnect),
                0x23 => self.inc_hl(),
                0x24 => self.inc_h(),
                0x25 => self.dec_h(),
                0x26 => self.ld_h_imm8(&operand),
                0x27 => self.daa(),
                0x28 => self.jr_z_imm8(&operand),
                0x29 => self.add_hl_hl(),
                0x2A => self.ld_a_hli(interconnect),
                0x2B => self.dec_hl(),
                0x2C => self.inc_l(),
                0x2D => self.dec_l(),
                0x2E => self.ld_l_imm8(&operand),
                0x2F => self.cpl(),
                0x30 => self.jr_nc_imm8(&operand),
                0x31 => self.ld_sp_imm16(&operand),
                0x32 => self.ld_hld_a(interconnect),
                0x33 => self.inc_sp(),
                0x34 => self.inc_hl_ptr(interconnect),
                0x35 => self.dec_hl_ptr(interconnect),
                0x36 => self.ld_hl_imm8(&operand, interconnect),
                0x37 => self.scf(),
                0x38 => self.jr_c_imm8(&operand),
                0x39 => self.add_hl_sp(),
                0x3A => self.ld_a_hld(interconnect),
                0x3B => self.dec_sp(),
                0x3C => self.inc_a(),
                0x3D => self.dec_a(),
                0x3E => self.ld_a_imm8(&operand),
                0x3F => self.ccf(),
                0x40 => (),
                0x41 => self.ld_b_c(),
                0x42 => self.ld_b_d(),
                0x43 => self.ld_b_e(),
                0x44 => self.ld_b_h(),
                0x45 => self.ld_b_l(),
                0x46 => self.ld_b_hl_ptr(interconnect),
                0x47 => self.ld_b_a(),
                0x48 => self.ld_c_b(),
                0x49 => (),
                0x4A => self.ld_c_d(),
                0x4B => self.ld_c_e(),
                0x4C => self.ld_c_h(),
                0x4D => self.ld_c_l(),
                0x4E => self.ld_c_hl_ptr(interconnect),
                0x4F => self.ld_c_a(),
                0x50 => self.ld_d_b(),
                0x51 => self.ld_d_c(),
                0x52 => (),
                0x53 => self.ld_d_e(),
                0x54 => self.ld_d_h(),
                0x55 => self.ld_d_l(),
                0x56 => self.ld_d_hl_ptr(interconnect),
                0x57 => self.ld_d_a(),
                0x58 => self.ld_e_b(),
                0x59 => self.ld_e_c(),
                0x5A => self.ld_e_d(),
                0x5B => (),
                0x5C => self.ld_e_h(),
                0x5D => self.ld_e_l(),
                0x5E => self.ld_e_hl(interconnect),
                0x5F => self.ld_e_a(),
                0x60 => self.ld_h_b(),
                0x61 => self.ld_h_c(),
                0x62 => self.ld_h_d(),
                0x63 => self.ld_h_e(),
                0x64 => (),
                0x65 => self.ld_h_l(),
                0x66 => self.ld_h_hl_ptr(interconnect),
                0x67 => self.ld_h_a(),
                0x68 => self.ld_l_b(),
                0x69 => self.ld_l_c(),
                0x6A => self.ld_l_d(),
                0x6B => self.ld_l_e(),
                0x6C => self.ld_l_h(),
                0x6D => (),
                0x6E => self.ld_l_hl(interconnect),
                0x6F => self.ld_l_a(),
                0x70 => self.ld_hl_ptr_b(interconnect),
                0x71 => self.ld_hl_ptr_c(interconnect),
                0x72 => self.ld_hl_ptr_d(interconnect),
                0x73 => self.ld_hl_ptr_e(interconnect),
                0x74 => self.ld_hl_ptr_h(interconnect),
                0x75 => self.ld_hl_ptr_l(interconnect),
                0x76 => self.halt(),
                0x77 => self.ld_hl_ptr_a(interconnect),
                0x78 => self.ld_a_b(),
                0x79 => self.ld_a_c(),
                0x7A => self.ld_a_d(),
                0x7B => self.ld_a_e(),
                0x7C => self.ld_a_h(),
                0x7D => self.ld_a_l(),
                0x7E => self.ld_a_hl(interconnect),
                0x7F => (),
                0x80 => self.add_a_b(),
                0x81 => self.add_a_c(),
                0x82 => self.add_a_d(),
                0x83 => self.add_a_e(),
                0x84 => self.add_a_h(),
                0x85 => self.add_a_l(),
                0x86 => self.add_a_hl_ptr(interconnect),
                0x87 => self.add_a_a(),
                0x88 => self.adc_a_b(),
                0x89 => self.adc_a_c(),
                0x8A => self.adc_a_d(),
                0x8B => self.adc_a_e(),
                0x8C => self.adc_a_h(),
                0x8D => self.adc_a_l(),
                0x8E => self.adc_a_hl_ptr(interconnect),
                0x8F => self.adc_a_a(),
                0x90 => self.sub_b(),
                0x91 => self.sub_c(),
                0x92 => self.sub_d(),
                0x93 => self.sub_e(),
                0x94 => self.sub_h(),
                0x95 => self.sub_l(),
                0x96 => self.sub_hl(interconnect),
                0x97 => self.sub_a(),
                0x98 => self.sbc_a_b(),
                0x99 => self.sbc_a_c(),
                0x9A => self.sbc_a_d(),
                0x9B => self.sbc_a_e(),
                0x9C => self.sbc_a_h(),
                0x9D => self.sbc_a_l(),
                0x9E => self.sbc_a_hl_ptr(interconnect),
                0x9F => self.sbc_a_a(),
                0xA0 => self.and_b(),
                0xA1 => self.and_c(),
                0xA2 => self.and_d(),
                0xA3 => self.and_e(),
                0xA4 => self.and_h(),
                0xA5 => self.and_l(),
                0xA6 => self.and_hl_ptr(interconnect),
                0xA7 => self.and_a(),
                0xA8 => self.xor_b(),
                0xA9 => self.xor_c(),
                0xAA => self.xor_d(),
                0xAB => self.xor_e(),
                0xAC => self.xor_h(),
                0xAD => self.xor_l(),
                0xAE => self.xor_hl_ptr(interconnect),
                0xAF => self.xor_a(),
                0xB0 => self.or_b(),
                0xB1 => self.or_c(),
                0xB2 => self.or_d(),
                0xB3 => self.or_e(),
                0xB4 => self.or_h(),
                0xB5 => self.or_l(),
                0xB6 => self.or_hl_ptr(interconnect),
                0xB7 => self.or_a(),
                0xB8 => self.cp_b(),
                0xB9 => self.cp_c(),
                0xBA => self.cp_d(),
                0xBB => self.cp_e(),
                0xBC => self.cp_h(),
                0xBD => self.cp_l(),
                0xBE => self.cp_hl_ptr(interconnect),
                0xBF => self.cp_a(),
                0xC0 => self.ret_nz(interconnect),
                0xC1 => self.pop_bc(interconnect),
                0xC2 => self.jp_nz_imm16(&operand),
                0xC3 => self.jp_imm16(&operand),
                0xC4 => self.call_nz_imm16(&operand, interconnect),
                0xC5 => self.push_bc(interconnect),
                0xC6 => self.add_a_imm8(&operand),
                0xC7 => self.call(0x00, interconnect),
                0xC8 => self.ret_z(interconnect),
                0xC9 => self.ret(interconnect),
                0xCA => self.jp_z_imm16(&operand),
                0xCB => {
                    cycles = self.handle_extended_opcode(interconnect)?;
                }
                0xCC => self.call_z_imm16(&operand, interconnect),
                0xCD => self.call(operand.unwrap_imm16(), interconnect),
                0xCE => self.adc_a_imm8(&operand),
                0xCF => self.call(0x08, interconnect),
                0xD0 => self.ret_nc(interconnect),
                0xD1 => self.pop_de(interconnect),
                0xD2 => self.jp_nc_imm16(&operand),
                0xD4 => self.call_nc_imm16(&operand, interconnect),
                0xD5 => self.push_de(interconnect),
                0xD6 => self.sub_imm8(&operand),
                0xD7 => self.call(0x10, interconnect),
                0xD8 => self.ret_c(interconnect),
                0xD9 => self.reti(interconnect),
                0xDA => self.jp_c_imm16(&operand),
                0xDC => self.call_c_imm16(&operand, interconnect),
                0xDE => self.sbc_a_imm8(&operand),
                0xDF => self.call(0x18, interconnect),
                0xE0 => self.ld_ff00_imm8_a(&operand, interconnect),
                0xE1 => self.pop_hl(interconnect),
                0xE2 => self.ld_ff00_c_a(interconnect),
                0xE5 => self.push_hl(interconnect),
                0xE6 => self.and_imm8(&operand),
                0xE7 => self.call(0x20, interconnect),
                0xE8 => self.add_sp_imm8(&operand),
                0xE9 => self.jp_hl(),
                0xEA => self.ld_imm16_a(&operand, interconnect),
                0xEE => self.xor_imm8(&operand),
                0xEF => self.call(0x28, interconnect),
                0xF0 => self.ld_a_ff00_imm8(&operand, interconnect),
                0xF1 => self.pop_af(interconnect),
                0xF2 => self.ld_a_c_ptr(interconnect),
                0xF3 => self.di(interconnect),
                0xF5 => self.push_af(interconnect),
                0xF6 => self.or_imm8(&operand),
                0xF7 => self.call(0x30, interconnect),
                0xF8 => self.ldhl_sp_imm8(&operand),
                0xF9 => self.ld_sp_hl(),
                0xFA => self.ld_a_imm16(&operand, interconnect),
                0xFB => self.ei(interconnect),
                0xFE => self.cp_imm8(&operand),
                0xFF => self.call(0x38, interconnect),
                _ => {
                    return Err(format!(
                        "Could not match opcode: {:02X} at offset: {:04X}",
                        opcode.code, self.registers.pc
                    ))
                }
            }

            return Ok(cycles);
        }

        Err(format!(
            "Unknown opcode: 0x{:02X} at offset: 0x{:04X}",
            byte, self.registers.pc
        ))
    }

    pub fn handle_extended_opcode(
        &mut self,
        interconnect: &mut Interconnect,
    ) -> Result<u8, String> {
        let byte = interconnect.read_u8(self.registers.pc);

        if let Some(opcode) = OpCode::from_byte(byte, true) {
            let operand = self.get_operand_from_opcode(interconnect, &opcode);

            // println!("Read 0x{:02X} from 0x{:04X}", byte, self.registers.pc);
            self.registers.pc += opcode.length;

            match opcode.code {
                0x00 => self.rlc_b(),
                0x01 => self.rlc_c(),
                0x02 => self.rlc_d(),
                0x03 => self.rlc_e(),
                0x04 => self.rlc_h(),
                0x05 => self.rlc_l(),
                0x06 => self.rlc_hl_ptr(interconnect),
                0x07 => self.rlc_a(),
                0x08 => self.rrc_b(),
                0x09 => self.rrc_c(),
                0x0A => self.rrc_d(),
                0x0B => self.rrc_e(),
                0x0C => self.rrc_h(),
                0x0D => self.rrc_l(),
                0x0E => self.rrc_hl_ptr(interconnect),
                0x0F => self.rrc_a(),
                0x10 => self.rl_b(),
                0x11 => self.rl_c(),
                0x12 => self.rl_d(),
                0x13 => self.rl_e(),
                0x14 => self.rl_h(),
                0x15 => self.rl_l(),
                0x16 => self.rl_hl_ptr(interconnect),
                0x17 => self.rl_a(),
                0x18 => self.rr_b(),
                0x19 => self.rr_c(),
                0x1A => self.rr_d(),
                0x1B => self.rr_e(),
                0x1C => self.rr_h(),
                0x1D => self.rr_l(),
                0x1E => self.rr_hl_ptr(interconnect),
                0x1F => self.rr_a(),
                0x20 => self.sla_b(),
                0x21 => self.sla_c(),
                0x22 => self.sla_d(),
                0x23 => self.sla_e(),
                0x24 => self.sla_h(),
                0x25 => self.sla_l(),
                0x26 => self.sla_hl_ptr(interconnect),
                0x27 => self.sla_a(),
                0x28 => self.sra_b(),
                0x29 => self.sra_c(),
                0x2A => self.sra_d(),
                0x2B => self.sra_e(),
                0x2C => self.sra_h(),
                0x2D => self.sra_l(),
                0x2E => self.sra_hl_ptr(interconnect),
                0x2F => self.sra_a(),
                0x30 => self.swap_b(),
                0x31 => self.swap_c(),
                0x32 => self.swap_d(),
                0x33 => self.swap_e(),
                0x34 => self.swap_h(),
                0x35 => self.swap_l(),
                0x36 => self.swap_hl_ptr(interconnect),
                0x37 => self.swap_a(),
                0x38 => self.srl_b(),
                0x39 => self.srl_c(),
                0x3A => self.srl_d(),
                0x3B => self.srl_e(),
                0x3C => self.srl_h(),
                0x3D => self.srl_l(),
                0x3E => self.srl_hl_ptr(interconnect),
                0x3F => self.srl_a(),
                0x40 => self.bit_0_b(),
                0x41 => self.bit_0_c(),
                0x42 => self.bit_0_d(),
                0x43 => self.bit_0_e(),
                0x44 => self.bit_0_h(),
                0x45 => self.bit_0_l(),
                0x46 => self.bit_0_hl_ptr(interconnect),
                0x47 => self.bit_0_a(),
                0x48 => self.bit_1_b(),
                0x49 => self.bit_1_c(),
                0x4A => self.bit_1_d(),
                0x4B => self.bit_1_e(),
                0x4C => self.bit_1_h(),
                0x4D => self.bit_1_l(),
                0x4E => self.bit_1_hl_ptr(interconnect),
                0x4F => self.bit_1_a(),
                0x50 => self.bit_2_b(),
                0x51 => self.bit_2_c(),
                0x52 => self.bit_2_d(),
                0x53 => self.bit_2_e(),
                0x54 => self.bit_2_h(),
                0x55 => self.bit_2_l(),
                0x56 => self.bit_2_hl_ptr(interconnect),
                0x57 => self.bit_2_a(),
                0x58 => self.bit_3_b(),
                0x59 => self.bit_3_c(),
                0x5A => self.bit_3_d(),
                0x5B => self.bit_3_e(),
                0x5C => self.bit_3_h(),
                0x5D => self.bit_3_l(),
                0x5E => self.bit_3_hl_ptr(interconnect),
                0x5F => self.bit_3_a(),
                0x60 => self.bit_4_b(),
                0x61 => self.bit_4_c(),
                0x62 => self.bit_4_d(),
                0x63 => self.bit_4_e(),
                0x64 => self.bit_4_h(),
                0x65 => self.bit_4_l(),
                0x66 => self.bit_4_hl_ptr(interconnect),
                0x67 => self.bit_4_a(),
                0x68 => self.bit_5_b(),
                0x69 => self.bit_5_c(),
                0x6A => self.bit_5_d(),
                0x6B => self.bit_5_e(),
                0x6C => self.bit_5_h(),
                0x6D => self.bit_5_l(),
                0x6E => self.bit_5_hl_ptr(interconnect),
                0x6F => self.bit_5_a(),
                0x70 => self.bit_6_b(),
                0x71 => self.bit_6_c(),
                0x72 => self.bit_6_d(),
                0x73 => self.bit_6_e(),
                0x74 => self.bit_6_h(),
                0x75 => self.bit_6_l(),
                0x76 => self.bit_6_hl_ptr(interconnect),
                0x77 => self.bit_6_a(),
                0x78 => self.bit_7_b(),
                0x79 => self.bit_7_c(),
                0x7A => self.bit_7_d(),
                0x7B => self.bit_7_e(),
                0x7C => self.bit_7_h(),
                0x7D => self.bit_7_l(),
                0x7E => self.bit_7_hl_ptr(interconnect),
                0x7F => self.bit_7_a(),

                0x80 => self.res_0_b(),
                0x81 => self.res_0_c(),
                0x82 => self.res_0_d(),
                0x83 => self.res_0_e(),
                0x84 => self.res_0_h(),
                0x85 => self.res_0_l(),
                0x86 => self.res_0_hl_ptr(interconnect),
                0x87 => self.res_0_a(),
                0x88 => self.res_1_b(),
                0x89 => self.res_1_c(),
                0x8A => self.res_1_d(),
                0x8B => self.res_1_e(),
                0x8C => self.res_1_h(),
                0x8D => self.res_1_l(),
                0x8E => self.res_1_hl_ptr(interconnect),
                0x8F => self.res_1_a(),
                0x90 => self.res_2_b(),
                0x91 => self.res_2_c(),
                0x92 => self.res_2_d(),
                0x93 => self.res_2_e(),
                0x94 => self.res_2_h(),
                0x95 => self.res_2_l(),
                0x96 => self.res_2_hl_ptr(interconnect),
                0x97 => self.res_2_a(),
                0x98 => self.res_3_b(),
                0x99 => self.res_3_c(),
                0x9A => self.res_3_d(),
                0x9B => self.res_3_e(),
                0x9C => self.res_3_h(),
                0x9D => self.res_3_l(),
                0x9E => self.res_3_hl_ptr(interconnect),
                0x9F => self.res_3_a(),
                0xA0 => self.res_4_b(),
                0xA1 => self.res_4_c(),
                0xA2 => self.res_4_d(),
                0xA3 => self.res_4_e(),
                0xA4 => self.res_4_h(),
                0xA5 => self.res_4_l(),
                0xA6 => self.res_4_hl_ptr(interconnect),
                0xA7 => self.res_4_a(),
                0xA8 => self.res_5_b(),
                0xA9 => self.res_5_c(),
                0xAA => self.res_5_d(),
                0xAB => self.res_5_e(),
                0xAC => self.res_5_h(),
                0xAD => self.res_5_l(),
                0xAE => self.res_5_hl_ptr(interconnect),
                0xAF => self.res_5_a(),
                0xB0 => self.res_6_b(),
                0xB1 => self.res_6_c(),
                0xB2 => self.res_6_d(),
                0xB3 => self.res_6_e(),
                0xB4 => self.res_6_h(),
                0xB5 => self.res_6_l(),
                0xB6 => self.res_6_hl_ptr(interconnect),
                0xB7 => self.res_6_a(),
                0xB8 => self.res_7_b(),
                0xB9 => self.res_7_c(),
                0xBA => self.res_7_d(),
                0xBB => self.res_7_e(),
                0xBC => self.res_7_h(),
                0xBD => self.res_7_l(),
                0xBE => self.res_7_hl_ptr(interconnect),
                0xBF => self.res_7_a(),

                0xC0 => self.set_0_b(),
                0xC1 => self.set_0_c(),
                0xC2 => self.set_0_d(),
                0xC3 => self.set_0_e(),
                0xC4 => self.set_0_h(),
                0xC5 => self.set_0_l(),
                0xC6 => self.set_0_hl_ptr(interconnect),
                0xC7 => self.set_0_a(),
                0xC8 => self.set_1_b(),
                0xC9 => self.set_1_c(),
                0xCA => self.set_1_d(),
                0xCB => self.set_1_e(),
                0xCC => self.set_1_h(),
                0xCD => self.set_1_l(),
                0xCE => self.set_1_hl_ptr(interconnect),
                0xCF => self.set_1_a(),
                0xD0 => self.set_2_b(),
                0xD1 => self.set_2_c(),
                0xD2 => self.set_2_d(),
                0xD3 => self.set_2_e(),
                0xD4 => self.set_2_h(),
                0xD5 => self.set_2_l(),
                0xD6 => self.set_2_hl_ptr(interconnect),
                0xD7 => self.set_2_a(),
                0xD8 => self.set_3_b(),
                0xD9 => self.set_3_c(),
                0xDA => self.set_3_d(),
                0xDB => self.set_3_e(),
                0xDC => self.set_3_h(),
                0xDD => self.set_3_l(),
                0xDE => self.set_3_hl_ptr(interconnect),
                0xDF => self.set_3_a(),
                0xE0 => self.set_4_b(),
                0xE1 => self.set_4_c(),
                0xE2 => self.set_4_d(),
                0xE3 => self.set_4_e(),
                0xE4 => self.set_4_h(),
                0xE5 => self.set_4_l(),
                0xE6 => self.set_4_hl_ptr(interconnect),
                0xE7 => self.set_4_a(),
                0xE8 => self.set_5_b(),
                0xE9 => self.set_5_c(),
                0xEA => self.set_5_d(),
                0xEB => self.set_5_e(),
                0xEC => self.set_5_h(),
                0xED => self.set_5_l(),
                0xEE => self.set_5_hl_ptr(interconnect),
                0xEF => self.set_5_a(),
                0xF0 => self.set_6_b(),
                0xF1 => self.set_6_c(),
                0xF2 => self.set_6_d(),
                0xF3 => self.set_6_e(),
                0xF4 => self.set_6_h(),
                0xF5 => self.set_6_l(),
                0xF6 => self.set_6_hl_ptr(interconnect),
                0xF7 => self.set_6_a(),
                0xF8 => self.set_7_b(),
                0xF9 => self.set_7_c(),
                0xFA => self.set_7_d(),
                0xFB => self.set_7_e(),
                0xFC => self.set_7_h(),
                0xFD => self.set_7_l(),
                0xFE => self.set_7_hl_ptr(interconnect),
                0xFF => self.set_7_a(),
                _ => {
                    return Err(format!(
                        "Could not match opcode: {:02X} at offset: {:04X}",
                        opcode.code, self.registers.pc
                    ))
                }
            }

            return Ok(opcode.cycles + 0x01);
        }

        Err(format!(
            "Unknown extended opcode: 0x{:02X} at offset: 0x{:04X}",
            byte, self.registers.pc
        ))
    }

    fn adc(&mut self, a: u8, b: u8) {
        let carry = if self.registers.flags.carry {
            0x01
        } else {
            0x00
        };

        let result = a.wrapping_add(b).wrapping_add(carry);

        self.registers.flags.half_carry = (a & 0x0F) + (b & 0x0F) + carry > 0x0F;
        self.registers.flags.negative = false;
        self.registers.flags.zero = result & 0xFF == 0x00;
        self.registers.flags.carry = (a as u16) + (b as u16) + (carry as u16) > 0xFF;

        self.registers.a = result as u8;
    }

    fn adc_a_a(&mut self) {
        let a = self.registers.a;
        self.adc(a, a);
    }

    fn adc_a_b(&mut self) {
        let a = self.registers.a;
        let b = self.registers.b;

        self.adc(a, b);
    }

    fn adc_a_c(&mut self) {
        let a = self.registers.a;
        let c = self.registers.c;

        self.adc(a, c);
    }

    fn adc_a_d(&mut self) {
        let a = self.registers.a;
        let d = self.registers.d;

        self.adc(a, d);
    }

    fn adc_a_e(&mut self) {
        let a = self.registers.a;
        let e = self.registers.e;

        self.adc(a, e);
    }

    fn adc_a_h(&mut self) {
        let a = self.registers.a;
        let h = self.registers.h;

        self.adc(a, h);
    }

    fn adc_a_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let a = self.registers.a;
        let val = interconnect.read_u8(self.registers.get_hl());

        self.adc(a, val);
    }

    fn adc_a_imm8(&mut self, operand: &Operand) {
        let a = self.registers.a;
        let val = operand.unwrap_imm8();

        self.adc(a, val);
    }

    fn adc_a_l(&mut self) {
        let a = self.registers.a;
        let l = self.registers.l;

        self.adc(a, l);
    }

    fn add_a_a(&mut self) {
        let a1 = self.registers.a;
        let a2 = self.registers.a;

        self.registers.a = a1.wrapping_add(a2);

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = ((a1 & 0x0F) + (a2 & 0x0F)) & 0x10 == 0x10;
        self.registers.flags.carry = (a1 as u16) + (a2 as u16) > 0xFF;
    }

    fn add_a_b(&mut self) {
        let a1 = self.registers.a;
        let a2 = self.registers.b;

        self.registers.a = a1.wrapping_add(a2);

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = ((a1 & 0x0F) + (a2 & 0x0F)) & 0x10 == 0x10;
        self.registers.flags.carry = (a1 as u16) + (a2 as u16) > 0xFF;
    }

    fn add_a_c(&mut self) {
        let a1 = self.registers.a;
        let a2 = self.registers.c;

        self.registers.a = a1.wrapping_add(a2);

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = ((a1 & 0x0F) + (a2 & 0x0F)) & 0x10 == 0x10;
        self.registers.flags.carry = (a1 as u16) + (a2 as u16) > 0xFF;
    }

    fn add_a_d(&mut self) {
        let a1 = self.registers.a;
        let a2 = self.registers.d;

        self.registers.a = a1.wrapping_add(a2);

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = ((a1 & 0x0F) + (a2 & 0x0F)) & 0x10 == 0x10;
        self.registers.flags.carry = (a1 as u16) + (a2 as u16) > 0xFF;
    }

    fn add_a_e(&mut self) {
        let a1 = self.registers.a;
        let a2 = self.registers.e;

        self.registers.a = a1.wrapping_add(a2);

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = ((a1 & 0x0F) + (a2 & 0x0F)) & 0x10 == 0x10;
        self.registers.flags.carry = (a1 as u16) + (a2 as u16) > 0xFF;
    }

    fn add_a_h(&mut self) {
        let a1 = self.registers.a;
        let a2 = self.registers.h;

        self.registers.a = a1.wrapping_add(a2);

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = ((a1 & 0x0F) + (a2 & 0x0F)) & 0x10 == 0x10;
        self.registers.flags.carry = (a1 as u16) + (a2 as u16) > 0xFF;
    }

    fn add_a_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let a1 = self.registers.a;
        let a2 = interconnect.read_u8(self.registers.get_hl());

        self.registers.a = a1.wrapping_add(a2);

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = ((a1 & 0x0F) + (a2 & 0x0F)) & 0x10 == 0x10;
        self.registers.flags.carry = (a1 as u16) + (a2 as u16) > 0xFF;
    }

    fn add_a_imm8(&mut self, operand: &Operand) {
        let a1 = self.registers.a;
        let val = operand.unwrap_imm8();

        self.registers.a = a1.wrapping_add(val);

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = ((a1 & 0x0F) + (val & 0x0F)) & 0x10 == 0x10;
        self.registers.flags.carry = (a1 as u16) + (val as u16) > 0xFF;
    }

    fn add_a_l(&mut self) {
        let a1 = self.registers.a;
        let a2 = self.registers.l;

        self.registers.a = a1.wrapping_add(a2);

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = ((a1 & 0x0F) + (a2 & 0x0F)) & 0x10 == 0x10;
        self.registers.flags.carry = (a1 as u16) + (a2 as u16) > 0xFF;
    }

    fn add_hl_bc(&mut self) {
        let hl = self.registers.get_hl();
        let bc = self.registers.get_bc();

        let r = hl.wrapping_add(bc);

        self.registers.set_hl(r);
        self.registers.flags.half_carry = ((hl & 0x07FF) + (bc & 0x07FF)) > 0x07FF;
        self.registers.flags.negative = false;
        self.registers.flags.carry = hl > 0xFFFF - bc;
    }

    fn add_hl_de(&mut self) {
        let hl = self.registers.get_hl();
        let de = self.registers.get_de();

        let r = hl.wrapping_add(de);

        self.registers.set_hl(r);
        self.registers.flags.half_carry = ((hl & 0x07FF) + (de & 0x07FF)) > 0x07FF;
        self.registers.flags.negative = false;
        self.registers.flags.carry = hl > 0xFFFF - de;
    }

    fn add_hl_hl(&mut self) {
        let hl = self.registers.get_hl();

        let r = hl.wrapping_add(hl);

        self.registers.set_hl(r);
        self.registers.flags.half_carry = ((hl & 0x07FF) + (hl & 0x07FF)) > 0x07FF;
        self.registers.flags.negative = false;
        self.registers.flags.carry = hl > 0xFFFF - hl;
    }

    fn add_hl_sp(&mut self) {
        let hl = self.registers.get_hl();
        let sp = self.registers.sp as u16;

        let r = hl.wrapping_add(sp);

        self.registers.set_hl(r);
        self.registers.flags.half_carry = ((hl & 0x07FF) + (sp & 0x07FF)) > 0x07FF;
        self.registers.flags.negative = false;
        self.registers.flags.carry = hl > 0xFFFF - sp;
    }

    fn add_sp_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8() as i8 as i16 as u16;
        let sp = self.registers.sp as u16;

        self.registers.flags.zero = false;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = (sp & 0x000F) + (val & 0x000F) > 0x000F;
        self.registers.flags.carry = (sp & 0x00FF) + (val & 0x00FF) > 0x00FF;

        self.registers.sp += val as usize;
    }

    fn and(&mut self, b: u8) {
        let r = self.registers.a & b;

        self.registers.a = r;
        self.registers.flags.zero = r == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = true;
        self.registers.flags.carry = false;
    }

    fn and_a(&mut self) {
        let a = self.registers.a;
        self.and(a);
    }

    fn and_b(&mut self) {
        let b = self.registers.b;
        self.and(b);
    }

    fn and_c(&mut self) {
        let c = self.registers.c;
        self.and(c);
    }

    fn and_d(&mut self) {
        let d = self.registers.d;
        self.and(d);
    }

    fn and_e(&mut self) {
        let e = self.registers.e;
        self.and(e);
    }

    fn and_h(&mut self) {
        let h = self.registers.h;
        self.and(h);
    }

    fn and_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        self.and(val);
    }

    fn and_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.and(val);
    }

    fn and_l(&mut self) {
        let l = self.registers.l;
        self.and(l);
    }

    fn bit(&mut self, b: u8, n: u8) {
        let shift = 0x01 << n;
        let bit = if b & shift == shift { true } else { false };

        self.registers.flags.zero = !bit;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = true;
    }

    fn bit_0_a(&mut self) {
        let a = self.registers.a;
        self.bit(a, 0x00);
    }

    fn bit_0_b(&mut self) {
        let b = self.registers.b;
        self.bit(b, 0x00);
    }

    fn bit_0_c(&mut self) {
        let c = self.registers.c;
        self.bit(c, 0x00);
    }

    fn bit_0_d(&mut self) {
        let d = self.registers.d;
        self.bit(d, 0x00);
    }

    fn bit_0_e(&mut self) {
        let e = self.registers.e;
        self.bit(e, 0x00);
    }

    fn bit_0_h(&mut self) {
        let h = self.registers.h;
        self.bit(h, 0x00);
    }

    fn bit_0_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        self.bit(val, 0x00);
    }

    fn bit_0_l(&mut self) {
        let l = self.registers.l;
        self.bit(l, 0x00);
    }

    fn bit_1_a(&mut self) {
        let a = self.registers.a;
        self.bit(a, 0x01);
    }

    fn bit_1_b(&mut self) {
        let b = self.registers.b;
        self.bit(b, 0x01);
    }

    fn bit_1_c(&mut self) {
        let c = self.registers.c;
        self.bit(c, 0x01);
    }

    fn bit_1_d(&mut self) {
        let d = self.registers.d;
        self.bit(d, 0x01);
    }

    fn bit_1_e(&mut self) {
        let e = self.registers.e;
        self.bit(e, 0x01);
    }

    fn bit_1_h(&mut self) {
        let h = self.registers.h;
        self.bit(h, 0x01);
    }

    fn bit_1_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        self.bit(val, 0x01);
    }

    fn bit_1_l(&mut self) {
        let l = self.registers.l;
        self.bit(l, 0x01);
    }

    fn bit_2_a(&mut self) {
        let a = self.registers.a;
        self.bit(a, 0x02);
    }

    fn bit_2_b(&mut self) {
        let b = self.registers.b;
        self.bit(b, 0x02);
    }

    fn bit_2_c(&mut self) {
        let c = self.registers.c;
        self.bit(c, 0x02);
    }

    fn bit_2_d(&mut self) {
        let d = self.registers.d;
        self.bit(d, 0x02);
    }

    fn bit_2_e(&mut self) {
        let e = self.registers.e;
        self.bit(e, 0x02);
    }

    fn bit_2_h(&mut self) {
        let h = self.registers.h;
        self.bit(h, 0x02);
    }

    fn bit_2_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        self.bit(val, 0x02);
    }

    fn bit_2_l(&mut self) {
        let l = self.registers.l;
        self.bit(l, 0x02);
    }

    fn bit_3_a(&mut self) {
        let a = self.registers.a;
        self.bit(a, 0x03);
    }

    fn bit_3_b(&mut self) {
        let b = self.registers.b;
        self.bit(b, 0x03);
    }

    fn bit_3_c(&mut self) {
        let c = self.registers.c;
        self.bit(c, 0x03);
    }

    fn bit_3_d(&mut self) {
        let d = self.registers.d;
        self.bit(d, 0x03);
    }

    fn bit_3_e(&mut self) {
        let e = self.registers.e;
        self.bit(e, 0x03);
    }

    fn bit_3_h(&mut self) {
        let h = self.registers.h;
        self.bit(h, 0x03);
    }

    fn bit_3_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        self.bit(val, 0x03);
    }

    fn bit_3_l(&mut self) {
        let l = self.registers.l;
        self.bit(l, 0x03);
    }

    fn bit_4_a(&mut self) {
        let a = self.registers.a;
        self.bit(a, 0x04);
    }

    fn bit_4_b(&mut self) {
        let b = self.registers.b;
        self.bit(b, 0x04);
    }

    fn bit_4_c(&mut self) {
        let c = self.registers.c;
        self.bit(c, 0x04);
    }

    fn bit_4_d(&mut self) {
        let d = self.registers.d;
        self.bit(d, 0x04);
    }

    fn bit_4_e(&mut self) {
        let e = self.registers.e;
        self.bit(e, 0x04);
    }

    fn bit_4_h(&mut self) {
        let h = self.registers.h;
        self.bit(h, 0x04);
    }

    fn bit_4_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        self.bit(val, 0x04);
    }

    fn bit_4_l(&mut self) {
        let l = self.registers.l;
        self.bit(l, 0x04);
    }

    fn bit_5_a(&mut self) {
        let a = self.registers.a;
        self.bit(a, 0x05);
    }

    fn bit_5_b(&mut self) {
        let b = self.registers.b;
        self.bit(b, 0x05);
    }

    fn bit_5_c(&mut self) {
        let c = self.registers.c;
        self.bit(c, 0x05);
    }

    fn bit_5_d(&mut self) {
        let d = self.registers.d;
        self.bit(d, 0x05);
    }

    fn bit_5_e(&mut self) {
        let e = self.registers.e;
        self.bit(e, 0x05);
    }

    fn bit_5_h(&mut self) {
        let h = self.registers.h;
        self.bit(h, 0x05);
    }

    fn bit_5_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        self.bit(val, 0x05);
    }

    fn bit_5_l(&mut self) {
        let l = self.registers.l;
        self.bit(l, 0x05);
    }

    fn bit_6_a(&mut self) {
        let a = self.registers.a;
        self.bit(a, 0x06);
    }

    fn bit_6_b(&mut self) {
        let b = self.registers.b;
        self.bit(b, 0x06);
    }

    fn bit_6_c(&mut self) {
        let c = self.registers.c;
        self.bit(c, 0x06);
    }

    fn bit_6_d(&mut self) {
        let d = self.registers.d;
        self.bit(d, 0x06);
    }

    fn bit_6_e(&mut self) {
        let e = self.registers.e;
        self.bit(e, 0x06);
    }

    fn bit_6_h(&mut self) {
        let h = self.registers.h;
        self.bit(h, 0x06);
    }

    fn bit_6_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        self.bit(val, 0x06);
    }

    fn bit_6_l(&mut self) {
        let l = self.registers.l;
        self.bit(l, 0x06);
    }

    fn bit_7_a(&mut self) {
        let a = self.registers.a;
        self.bit(a, 0x07);
    }

    fn bit_7_b(&mut self) {
        let b = self.registers.b;
        self.bit(b, 0x07);
    }

    fn bit_7_c(&mut self) {
        let c = self.registers.c;
        self.bit(c, 0x07);
    }

    fn bit_7_d(&mut self) {
        let d = self.registers.d;
        self.bit(d, 0x07);
    }

    fn bit_7_e(&mut self) {
        let e = self.registers.e;
        self.bit(e, 0x07);
    }

    fn bit_7_h(&mut self) {
        let h = self.registers.h;
        self.bit(h, 0x07);
    }

    fn bit_7_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        self.bit(val, 0x07);
    }

    fn bit_7_l(&mut self) {
        let l = self.registers.l;
        self.bit(l, 0x07);
    }

    fn call(&mut self, addr: u16, interconnect: &mut Interconnect) {
        self.registers.sp -= 0x02;
        interconnect.write_u16(self.registers.sp as u16, self.registers.pc);
        self.registers.pc = addr;
    }

    fn call_c_imm16(&mut self, operand: &Operand, interconnect: &mut Interconnect) {
        if self.registers.flags.carry {
            let addr = operand.unwrap_imm16();
            self.call(addr, interconnect);
        }
    }

    fn call_nc_imm16(&mut self, operand: &Operand, interconnect: &mut Interconnect) {
        if self.registers.flags.carry == false {
            let addr = operand.unwrap_imm16();
            self.call(addr, interconnect);
        }
    }

    fn call_nz_imm16(&mut self, operand: &Operand, interconnect: &mut Interconnect) {
        if self.registers.flags.zero == false {
            let addr = operand.unwrap_imm16();
            self.call(addr, interconnect);
        }
    }

    fn call_z_imm16(&mut self, operand: &Operand, interconnect: &mut Interconnect) {
        if self.registers.flags.zero {
            let addr = operand.unwrap_imm16();
            self.call(addr, interconnect);
        }
    }

    fn ccf(&mut self) {
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = !self.registers.flags.carry;
    }

    fn cp(&mut self, b: u8) {
        let r = self.registers.a.wrapping_sub(b);

        self.registers.flags.zero = r == 0x00;
        self.registers.flags.negative = true;
        self.registers.flags.half_carry = (self.registers.a & 0x0F) < (b & 0x0F);
        self.registers.flags.carry = self.registers.a < b;
    }

    fn cp_a(&mut self) {
        let a = self.registers.a;
        self.cp(a);
    }

    fn cp_b(&mut self) {
        let b = self.registers.b;
        self.cp(b);
    }

    fn cp_c(&mut self) {
        let c = self.registers.c;
        self.cp(c);
    }

    fn cp_d(&mut self) {
        let d = self.registers.d;
        self.cp(d);
    }

    fn cp_e(&mut self) {
        let e = self.registers.e;
        self.cp(e);
    }

    fn cp_h(&mut self) {
        let h = self.registers.h;
        self.cp(h);
    }

    fn cp_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        self.cp(val);
    }

    fn cp_l(&mut self) {
        let l = self.registers.l;
        self.cp(l);
    }

    fn cp_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.cp(val);
    }

    fn cpl(&mut self) {
        self.registers.a = !self.registers.a as u8;
        self.registers.flags.negative = true;
        self.registers.flags.half_carry = true;
    }

    fn daa(&mut self) {
        let mut correction_factor: u8 = 0x00;
        let subtraction = self.registers.flags.negative;

        if (self.registers.a > 0x99 && !subtraction) || self.registers.flags.carry {
            correction_factor |= 0x60;
            self.registers.flags.carry = true;
        }

        if ((self.registers.a & 0x0F) > 0x09 && !subtraction) || self.registers.flags.half_carry {
            correction_factor |= 0x06;
        }

        if subtraction {
            self.registers.a -= correction_factor;
        } else {
            self.registers.a += correction_factor;
        }

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.half_carry = false;
    }

    fn dec_a(&mut self) {
        let r = self.registers.a;
        self.registers.a = r.wrapping_sub(0x01);

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = true;
        self.registers.flags.half_carry = (r & 0x0F) == 0x00;
    }

    fn dec_b(&mut self) {
        let r = self.registers.b;
        self.registers.b = r.wrapping_sub(0x01);

        self.registers.flags.zero = self.registers.b == 0x00;
        self.registers.flags.negative = true;
        self.registers.flags.half_carry = (r & 0x0F) == 0x00;
    }

    fn dec_bc(&mut self) {
        let val = self.registers.get_bc().wrapping_sub(0x01);
        self.registers.set_bc(val);
    }

    fn dec_de(&mut self) {
        let val = self.registers.get_de().wrapping_sub(0x01);
        self.registers.set_de(val);
    }

    fn dec_c(&mut self) {
        let r = self.registers.c;
        self.registers.c = r.wrapping_sub(0x01);

        self.registers.flags.zero = self.registers.c == 0x00;
        self.registers.flags.negative = true;
        self.registers.flags.half_carry = (r & 0x0F) == 0x00;
    }

    fn dec_d(&mut self) {
        let r = self.registers.d;
        self.registers.d = r.wrapping_sub(0x01);

        self.registers.flags.zero = self.registers.d == 0x00;
        self.registers.flags.negative = true;
        self.registers.flags.half_carry = (r & 0x0F) == 0x00;
    }

    fn dec_e(&mut self) {
        let r = self.registers.e;
        self.registers.e = r.wrapping_sub(0x01);

        self.registers.flags.zero = self.registers.e == 0x00;
        self.registers.flags.negative = true;
        self.registers.flags.half_carry = (r & 0x0F) == 0x00;
    }

    fn dec_h(&mut self) {
        let r = self.registers.h;
        self.registers.h = r.wrapping_sub(0x01);

        self.registers.flags.zero = self.registers.h == 0x00;
        self.registers.flags.negative = true;
        self.registers.flags.half_carry = (r & 0x0F) == 0x00;
    }

    fn dec_hl(&mut self) {
        let val = self.registers.get_hl().wrapping_sub(0x01);
        self.registers.set_hl(val);
    }

    fn dec_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        let result = val.wrapping_sub(0x01);
        interconnect.write_u8(self.registers.get_hl(), result);

        self.registers.flags.zero = result == 0x00;
        self.registers.flags.negative = true;
        self.registers.flags.half_carry = (val & 0x0F) == 0x00;
    }

    fn dec_l(&mut self) {
        let r = self.registers.l;
        self.registers.l = r.wrapping_sub(0x01);

        self.registers.flags.zero = self.registers.l == 0x00;
        self.registers.flags.negative = true;
        self.registers.flags.half_carry = (r & 0x0F) == 0x00;
    }

    fn dec_sp(&mut self) {
        self.registers.sp = self.registers.sp.wrapping_sub(0x01);
    }

    fn di(&mut self, interconnect: &mut Interconnect) {
        interconnect.irq.enabled = false;
    }

    fn ei(&mut self, interconnect: &mut Interconnect) {
        interconnect.irq.enabled = true;
    }

    fn halt(&mut self) {
        self.halted = true;
    }

    fn inc_a(&mut self) {
        let r = self.registers.a;
        self.registers.a = r.wrapping_add(0x01);

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = (r & 0x0F) + 0x01 > 0x0F;
    }

    fn inc_b(&mut self) {
        let r = self.registers.b;
        self.registers.b = r.wrapping_add(0x01);

        self.registers.flags.zero = self.registers.b == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = (r & 0x0F) + 0x01 > 0x0F;
    }

    fn inc_bc(&mut self) {
        let val = self.registers.get_bc();
        self.registers.set_bc(val + 0x01);
    }

    fn inc_c(&mut self) {
        let r = self.registers.c;
        self.registers.c = r.wrapping_add(0x01);

        self.registers.flags.zero = self.registers.c == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = (r & 0x0F) + 0x01 > 0x0F;
    }

    fn inc_d(&mut self) {
        let r = self.registers.d;
        self.registers.d = r.wrapping_add(0x01);

        self.registers.flags.zero = self.registers.d == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = (r & 0x0F) + 0x01 > 0x0F;
    }

    fn inc_de(&mut self) {
        let val = self.registers.get_de();
        self.registers.set_de(val + 0x01);
    }

    fn inc_e(&mut self) {
        let r = self.registers.e;
        self.registers.e = r.wrapping_add(0x01);

        self.registers.flags.zero = self.registers.e == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = (r & 0x0F) + 0x01 > 0x0F;
    }

    fn inc_h(&mut self) {
        let r = self.registers.h;
        self.registers.h = r.wrapping_add(0x01);

        self.registers.flags.zero = self.registers.h == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = (r & 0x0F) + 0x01 > 0x0F;
    }

    fn inc_hl(&mut self) {
        let val = self.registers.get_hl();
        self.registers.set_hl(val + 0x01);
    }

    fn inc_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        let result = val.wrapping_add(0x01);
        interconnect.write_u8(self.registers.get_hl(), result);

        self.registers.flags.zero = result == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = (val & 0x0F) + 0x01 > 0x0F;
    }

    fn inc_l(&mut self) {
        let r = self.registers.l;
        self.registers.l = r.wrapping_add(0x01);

        self.registers.flags.zero = self.registers.l == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = (r & 0x0F) + 0x01 > 0x0F;
    }

    fn inc_sp(&mut self) {
        self.registers.sp += 0x01;
    }

    fn jp_c_imm16(&mut self, operand: &Operand) {
        if self.registers.flags.carry {
            self.jp_imm16(operand);
        }
    }

    fn jp_hl(&mut self) {
        self.registers.pc = self.registers.get_hl();
    }

    fn jp_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.relative_jump(val);
    }

    fn jp_imm16(&mut self, operand: &Operand) {
        let addr = operand.unwrap_imm16();
        self.registers.set_pc(addr);
    }

    fn jp_nc_imm16(&mut self, operand: &Operand) {
        if self.registers.flags.carry == false {
            self.jp_imm16(operand);
        }
    }

    fn jp_nz_imm16(&mut self, operand: &Operand) {
        if !self.registers.flags.zero {
            self.jp_imm16(operand);
        }
    }

    fn jp_z_imm16(&mut self, operand: &Operand) {
        if self.registers.flags.zero {
            self.jp_imm16(operand);
        }
    }

    fn jr_c_imm8(&mut self, operand: &Operand) {
        let offset = operand.unwrap_imm8();

        if self.registers.flags.carry {
            self.relative_jump(offset);
        }
    }

    fn jr_nc_imm8(&mut self, operand: &Operand) {
        let offset = operand.unwrap_imm8();

        if !self.registers.flags.carry {
            self.relative_jump(offset);
        }
    }

    fn jr_nz_imm8(&mut self, operand: &Operand) {
        let offset = operand.unwrap_imm8();

        if self.registers.flags.zero == false {
            self.relative_jump(offset);
        }
    }

    fn jr_z_imm8(&mut self, operand: &Operand) {
        let offset = operand.unwrap_imm8();

        if self.registers.flags.zero {
            self.relative_jump(offset);
        }
    }

    fn ld_a_b(&mut self) {
        self.registers.a = self.registers.b;
    }

    fn ld_a_bc(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_bc();
        let val = interconnect.read_u8(addr);
        self.registers.a = val;
    }

    fn ld_a_c(&mut self) {
        self.registers.a = self.registers.c;
    }

    fn ld_a_c_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(0xFF00 + self.registers.c as u16);
        self.registers.a = val;
    }

    fn ld_a_d(&mut self) {
        self.registers.a = self.registers.d;
    }

    fn ld_a_de(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_de();
        let val = interconnect.read_u8(addr);
        self.registers.a = val;
    }

    fn ld_a_e(&mut self) {
        self.registers.a = self.registers.e;
    }

    fn ld_a_ff00_imm8(&mut self, operand: &Operand, interconnect: &mut Interconnect) {
        let offset = operand.unwrap_imm8();
        let addr = 0xFF00 as u16 + offset as u16;
        self.registers.a = interconnect.read_u8(addr);
    }

    fn ld_a_h(&mut self) {
        self.registers.a = self.registers.h;
    }

    fn ld_a_hl(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        let val = interconnect.read_u8(addr);
        self.registers.a = val;
    }

    fn ld_a_hld(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        self.registers.a = interconnect.read_u8(addr);
        self.registers.set_hl(addr - 0x01);
    }

    fn ld_a_hli(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        let val = interconnect.read_u8(addr);
        self.registers.a = val;
        self.registers.set_hl(addr + 0x01);
    }

    fn ld_a_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.registers.a = val;
    }

    fn ld_a_imm16(&mut self, operand: &Operand, interconnect: &mut Interconnect) {
        let addr = operand.unwrap_imm16();
        let val = interconnect.read_u8(addr);
        self.registers.a = val;
    }

    fn ld_a_l(&mut self) {
        self.registers.a = self.registers.l;
    }

    fn ld_b_a(&mut self) {
        self.registers.b = self.registers.a;
    }

    fn ld_b_c(&mut self) {
        self.registers.b = self.registers.c;
    }

    fn ld_b_d(&mut self) {
        self.registers.b = self.registers.d;
    }

    fn ld_b_e(&mut self) {
        self.registers.b = self.registers.e;
    }

    fn ld_b_h(&mut self) {
        self.registers.b = self.registers.h;
    }

    fn ld_b_l(&mut self) {
        self.registers.b = self.registers.l;
    }

    fn ld_b_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        let val = interconnect.read_u8(addr);
        self.registers.b = val;
    }

    fn ld_bc_a(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_bc();
        interconnect.write_u8(addr, self.registers.a);
    }

    fn ld_bc_imm16(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm16();
        self.registers.set_bc(val);
    }

    fn ld_b_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.registers.b = val;
    }

    fn ld_c_a(&mut self) {
        self.registers.c = self.registers.a;
    }

    fn ld_c_b(&mut self) {
        self.registers.c = self.registers.b;
    }

    fn ld_c_d(&mut self) {
        self.registers.c = self.registers.d;
    }

    fn ld_c_e(&mut self) {
        self.registers.c = self.registers.e;
    }

    fn ld_c_h(&mut self) {
        self.registers.c = self.registers.h;
    }

    fn ld_c_l(&mut self) {
        self.registers.c = self.registers.l;
    }

    fn ld_c_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        let val = interconnect.read_u8(addr);
        self.registers.c = val;
    }

    fn ld_c_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.registers.c = val;
    }

    fn ld_d_a(&mut self) {
        self.registers.d = self.registers.a;
    }

    fn ld_d_b(&mut self) {
        self.registers.d = self.registers.b;
    }

    fn ld_d_c(&mut self) {
        self.registers.d = self.registers.c;
    }

    fn ld_d_e(&mut self) {
        self.registers.d = self.registers.e;
    }

    fn ld_d_h(&mut self) {
        self.registers.d = self.registers.h;
    }

    fn ld_d_l(&mut self) {
        self.registers.d = self.registers.l;
    }

    fn ld_d_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        let val = interconnect.read_u8(addr);
        self.registers.d = val;
    }

    fn ld_d_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.registers.d = val;
    }

    fn ld_de_a(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_de();
        interconnect.write_u8(addr, self.registers.a);
    }

    fn ld_de_imm16(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm16();
        self.registers.set_de(val);
    }

    fn ld_e_a(&mut self) {
        self.registers.e = self.registers.a;
    }

    fn ld_e_b(&mut self) {
        self.registers.e = self.registers.b;
    }

    fn ld_e_c(&mut self) {
        self.registers.e = self.registers.c;
    }

    fn ld_e_d(&mut self) {
        self.registers.e = self.registers.d;
    }

    fn ld_e_h(&mut self) {
        self.registers.e = self.registers.h;
    }

    fn ld_e_hl(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        let val = interconnect.read_u8(addr);
        self.registers.e = val;
    }

    fn ld_e_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.registers.e = val;
    }

    fn ld_e_l(&mut self) {
        self.registers.e = self.registers.l;
    }

    fn ld_ff00_imm8_a(&mut self, operand: &Operand, interconnect: &mut Interconnect) {
        let offset = operand.unwrap_imm8();
        let addr = 0xFF00 as u16 + offset as u16;
        interconnect.write_u8(addr, self.registers.a);
    }

    fn ld_ff00_c_a(&mut self, interconnect: &mut Interconnect) {
        let addr = 0xFF00 as u16 + self.registers.c as u16;
        interconnect.write_u8(addr, self.registers.a);
    }

    fn ldhl_sp_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8() as i8 as i16 as u16;
        let sp = self.registers.sp as u16;

        self.registers.flags.zero = false;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = (sp & 0x000F) + (val & 0x000F) > 0x000F;
        self.registers.flags.carry = (sp & 0x00FF) + (val & 0x00FF) > 0x00FF;

        self.registers.set_hl(sp.wrapping_add(val));
    }

    fn ld_h_a(&mut self) {
        self.registers.h = self.registers.a;
    }

    fn ld_h_b(&mut self) {
        self.registers.h = self.registers.b;
    }

    fn ld_h_c(&mut self) {
        self.registers.h = self.registers.c;
    }

    fn ld_h_d(&mut self) {
        self.registers.h = self.registers.d;
    }

    fn ld_h_e(&mut self) {
        self.registers.h = self.registers.e;
    }

    fn ld_h_l(&mut self) {
        self.registers.h = self.registers.l;
    }

    fn ld_h_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        self.registers.h = interconnect.read_u8(self.registers.get_hl());
    }

    fn ld_h_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.registers.h = val;
    }

    fn ld_hl_ptr_a(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        interconnect.write_u8(addr, self.registers.a);
    }

    fn ld_hl_ptr_b(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        interconnect.write_u8(addr, self.registers.b);
    }

    fn ld_hl_ptr_c(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        interconnect.write_u8(addr, self.registers.c);
    }

    fn ld_hl_ptr_d(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        interconnect.write_u8(addr, self.registers.d);
    }

    fn ld_hl_ptr_e(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        interconnect.write_u8(addr, self.registers.e);
    }

    fn ld_hl_ptr_h(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        interconnect.write_u8(addr, self.registers.h);
    }

    fn ld_hl_ptr_l(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        interconnect.write_u8(addr, self.registers.l);
    }

    fn ld_hl_imm8(&mut self, operand: &Operand, interconnect: &mut Interconnect) {
        let val = operand.unwrap_imm8();
        let addr = self.registers.get_hl();

        interconnect.write_u8(addr, val);
    }

    fn ld_hl_imm16(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm16();
        self.registers.set_hl(val);
    }

    fn ld_hld_a(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        interconnect.write_u8(addr, self.registers.a);
        self.registers.set_hl(addr - 0x01);
    }

    fn ld_hli_a(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        interconnect.write_u8(addr, self.registers.a);
        self.registers.set_hl(addr + 0x01);
    }

    fn ld_imm16_a(&mut self, operand: &Operand, interconnect: &mut Interconnect) {
        let addr = operand.unwrap_imm16();
        interconnect.write_u8(addr, self.registers.a);
    }

    fn ld_imm16_ptr_sp(&mut self, operand: &Operand, interconnect: &mut Interconnect) {
        let addr = operand.unwrap_imm16();
        interconnect.write_u16(addr, self.registers.sp as u16);
    }

    fn ld_l_a(&mut self) {
        self.registers.l = self.registers.a;
    }

    fn ld_l_b(&mut self) {
        self.registers.l = self.registers.b;
    }

    fn ld_l_c(&mut self) {
        self.registers.l = self.registers.c;
    }

    fn ld_l_d(&mut self) {
        self.registers.l = self.registers.d;
    }

    fn ld_l_e(&mut self) {
        self.registers.l = self.registers.e;
    }

    fn ld_l_h(&mut self) {
        self.registers.l = self.registers.h;
    }

    fn ld_l_hl(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        let val = interconnect.read_u8(addr);
        self.registers.l = val;
    }

    fn ld_l_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.registers.l = val;
    }

    fn ld_sp_hl(&mut self) {
        self.registers.sp = self.registers.get_hl() as usize;
    }

    fn ld_sp_imm16(&mut self, operand: &Operand) {
        let addr = operand.unwrap_imm16();
        self.registers.sp = addr as usize;
    }

    fn or_a(&mut self) {
        self.registers.a |= self.registers.a;

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = false;
    }

    fn or_b(&mut self) {
        self.registers.a |= self.registers.b;

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = false;
    }

    fn or_c(&mut self) {
        self.registers.a |= self.registers.c;

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = false;
    }

    fn or_d(&mut self) {
        self.registers.a |= self.registers.d;

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = false;
    }

    fn or_e(&mut self) {
        self.registers.a |= self.registers.e;

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = false;
    }

    fn or_h(&mut self) {
        self.registers.a |= self.registers.h;

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = false;
    }

    fn or_l(&mut self) {
        self.registers.a |= self.registers.l;

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = false;
    }

    fn or_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        self.registers.a |= val;

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = false;
    }

    fn or_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.registers.a |= val;

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = false;
    }

    fn pop_af(&mut self, interconnect: &mut Interconnect) {
        let addr = interconnect.read_u16(self.registers.sp as u16);
        self.registers.sp += 0x02;
        self.registers.set_af(addr);
    }

    fn pop_bc(&mut self, interconnect: &mut Interconnect) {
        let addr = interconnect.read_u16(self.registers.sp as u16);
        self.registers.sp += 0x02;
        self.registers.set_bc(addr);
    }

    fn pop_de(&mut self, interconnect: &mut Interconnect) {
        let addr = interconnect.read_u16(self.registers.sp as u16);
        self.registers.sp += 0x02;
        self.registers.set_de(addr);
    }

    fn pop_hl(&mut self, interconnect: &mut Interconnect) {
        let addr = interconnect.read_u16(self.registers.sp as u16);
        self.registers.sp += 0x02;
        self.registers.set_hl(addr);
    }

    fn push_af(&mut self, interconnect: &mut Interconnect) {
        let val = self.registers.get_af();
        self.registers.sp -= 0x02;
        interconnect.write_u16(self.registers.sp as u16, val);
    }

    fn push_bc(&mut self, interconnect: &mut Interconnect) {
        let val = self.registers.get_bc();
        self.registers.sp -= 0x02;
        interconnect.write_u16(self.registers.sp as u16, val);
    }

    fn push_de(&mut self, interconnect: &mut Interconnect) {
        let val = self.registers.get_de();
        self.registers.sp -= 0x02;
        interconnect.write_u16(self.registers.sp as u16, val);
    }

    fn push_hl(&mut self, interconnect: &mut Interconnect) {
        let val = self.registers.get_hl();
        self.registers.sp -= 0x02;
        interconnect.write_u16(self.registers.sp as u16, val);
    }

    fn res(&mut self, mut b: u8, n: u8) -> u8 {
        let shift = 0x01 << n;
        b &= !shift;

        b
    }

    fn res_0_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.res(a, 0x00);
    }

    fn res_0_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.res(b, 0x00);
    }

    fn res_0_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.res(c, 0x00);
    }

    fn res_0_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.res(d, 0x00);
    }

    fn res_0_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.res(e, 0x00);
    }

    fn res_0_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.res(h, 0x00);
    }

    fn res_0_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.res(val, 0x00));
    }

    fn res_0_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.res(l, 0x00);
    }

    fn res_1_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.res(a, 0x01);
    }

    fn res_1_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.res(b, 0x01);
    }

    fn res_1_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.res(c, 0x01);
    }

    fn res_1_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.res(d, 0x01);
    }

    fn res_1_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.res(e, 0x01);
    }

    fn res_1_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.res(h, 0x01);
    }

    fn res_1_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.res(val, 0x01));
    }

    fn res_1_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.res(l, 0x01);
    }

    fn res_2_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.res(a, 0x02);
    }

    fn res_2_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.res(b, 0x02);
    }

    fn res_2_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.res(c, 0x02);
    }

    fn res_2_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.res(d, 0x02);
    }

    fn res_2_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.res(e, 0x02);
    }

    fn res_2_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.res(h, 0x02);
    }

    fn res_2_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.res(val, 0x02));
    }

    fn res_2_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.res(l, 0x02);
    }

    fn res_3_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.res(a, 0x03);
    }

    fn res_3_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.res(b, 0x03);
    }

    fn res_3_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.res(c, 0x03);
    }

    fn res_3_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.res(d, 0x03);
    }

    fn res_3_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.res(e, 0x03);
    }

    fn res_3_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.res(h, 0x03);
    }

    fn res_3_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.res(val, 0x03));
    }

    fn res_3_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.res(l, 0x03);
    }

    fn res_4_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.res(a, 0x04);
    }

    fn res_4_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.res(b, 0x04);
    }

    fn res_4_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.res(c, 0x04);
    }

    fn res_4_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.res(d, 0x04);
    }

    fn res_4_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.res(e, 0x04);
    }

    fn res_4_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.res(h, 0x04);
    }

    fn res_4_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.res(val, 0x04));
    }

    fn res_4_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.res(l, 0x04);
    }

    fn res_5_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.res(a, 0x05);
    }

    fn res_5_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.res(b, 0x05);
    }

    fn res_5_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.res(c, 0x05);
    }

    fn res_5_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.res(d, 0x05);
    }

    fn res_5_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.res(e, 0x05);
    }

    fn res_5_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.res(h, 0x05);
    }

    fn res_5_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.res(val, 0x05));
    }

    fn res_5_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.res(l, 0x05);
    }

    fn res_6_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.res(a, 0x06);
    }

    fn res_6_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.res(b, 0x06);
    }

    fn res_6_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.res(c, 0x06);
    }

    fn res_6_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.res(d, 0x06);
    }

    fn res_6_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.res(e, 0x06);
    }

    fn res_6_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.res(h, 0x06);
    }

    fn res_6_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.res(val, 0x06));
    }

    fn res_6_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.res(l, 0x06);
    }

    fn res_7_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.res(a, 0x07);
    }

    fn res_7_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.res(b, 0x07);
    }

    fn res_7_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.res(c, 0x07);
    }

    fn res_7_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.res(d, 0x07);
    }

    fn res_7_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.res(e, 0x07);
    }

    fn res_7_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.res(h, 0x07);
    }

    fn res_7_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.res(val, 0x07));
    }

    fn res_7_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.res(l, 0x07);
    }

    fn ret(&mut self, interconnect: &mut Interconnect) {
        let addr = interconnect.read_u16(self.registers.sp as u16);
        self.registers.sp += 0x02;
        self.registers.pc = addr;
    }

    fn reti(&mut self, interconnect: &mut Interconnect) {
        let addr = interconnect.read_u16(self.registers.sp as u16);
        self.registers.sp += 0x02;
        self.registers.pc = addr;

        interconnect.irq.enabled = true;
    }

    fn ret_c(&mut self, interconnect: &mut Interconnect) {
        if self.registers.flags.carry {
            let addr = interconnect.read_u16(self.registers.sp as u16);
            self.registers.sp += 0x02;
            self.registers.pc = addr;
        }
    }

    fn ret_nc(&mut self, interconnect: &mut Interconnect) {
        if self.registers.flags.carry == false {
            let addr = interconnect.read_u16(self.registers.sp as u16);
            self.registers.sp += 0x02;
            self.registers.pc = addr;
        }
    }

    fn ret_nz(&mut self, interconnect: &mut Interconnect) {
        if self.registers.flags.zero == false {
            let addr = interconnect.read_u16(self.registers.sp as u16);
            self.registers.sp += 0x02;
            self.registers.pc = addr;
        }
    }

    fn ret_z(&mut self, interconnect: &mut Interconnect) {
        if self.registers.flags.zero {
            let addr = interconnect.read_u16(self.registers.sp as u16);
            self.registers.sp += 0x02;
            self.registers.pc = addr;
        }
    }

    fn rla(&mut self) {
        let original_carry = if self.registers.flags.carry {
            0x01
        } else {
            0x00
        };
        self.registers.flags.carry = self.registers.a & 0x80 == 0x80;
        self.registers.a = (self.registers.a << 0x01) | original_carry;

        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.zero = false;
    }

    fn rl(&mut self, mut b: u8) -> u8 {
        let original_carry = if self.registers.flags.carry {
            0x01
        } else {
            0x00
        };
        self.registers.flags.carry = b & 0x80 == 0x80;
        b = (b << 0x01) | original_carry;

        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.zero = b == 0x00;

        b
    }

    fn rl_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.rl(a);
    }

    fn rl_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.rl(b);
    }

    fn rl_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.rl(c);
    }

    fn rl_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.rl(d);
    }

    fn rl_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.rl(e);
    }

    fn rl_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.rl(h);
    }

    fn rl_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.rl(val));
    }

    fn rl_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.rl(l);
    }

    fn rlc(&mut self, mut b: u8) -> u8 {
        let carry = if b & 0x80 == 0x80 { true } else { false };
        b = (b << 0x01) | if carry { 0x01 } else { 0x00 };

        self.registers.flags.zero = b == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = carry;

        b
    }

    fn rlc_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.rlc(a);
    }

    fn rlc_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.rlc(b);
    }

    fn rlc_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.rlc(c);
    }

    fn rlc_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.rlc(d);
    }

    fn rlc_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.rlc(e);
    }

    fn rlc_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.rlc(h);
    }

    fn rlc_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.rlc(val));
    }

    fn rlc_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.rlc(l);
    }

    fn rlca(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.rlc(a);
        self.registers.flags.zero = false;
    }

    fn rr(&mut self, mut b: u8) -> u8 {
        let original_carry = if self.registers.flags.carry {
            0x80
        } else {
            0x00
        };
        self.registers.flags.carry = b & 0x01 == 0x01;
        b = (b >> 0x01) | original_carry;

        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.zero = b == 0x00;

        b
    }

    fn rr_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.rr(a);
    }

    fn rr_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.rr(b);
    }

    fn rr_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.rr(c);
    }

    fn rr_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.rr(d);
    }

    fn rr_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.rr(e);
    }

    fn rr_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.rr(h);
    }

    fn rr_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.rr(val));
    }

    fn rr_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.rr(l);
    }

    fn rra(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.rr(a);
        self.registers.flags.zero = false;
    }

    fn rrc(&mut self, mut b: u8) -> u8 {
        let carry = if b & 0x01 == 0x01 { true } else { false };
        b = (b >> 0x01) | if carry { 0x80 } else { 0x00 };

        self.registers.flags.zero = b == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = carry;

        b
    }

    fn rrc_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.rrc(a);
    }

    fn rrc_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.rrc(b);
    }

    fn rrc_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.rrc(c);
    }

    fn rrc_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.rrc(d);
    }

    fn rrc_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.rrc(e);
    }

    fn rrc_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.rrc(h);
    }

    fn rrc_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.rrc(val));
    }

    fn rrc_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.rrc(l);
    }

    fn rrca(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.rrc(a);
        self.registers.flags.zero = false;
    }

    fn sbc(&mut self, b: u8) {
        let a = self.registers.a;
        let carry = if self.registers.flags.carry {
            0x01
        } else {
            0x00
        };

        let result = a.wrapping_sub(b).wrapping_sub(carry);

        self.registers.flags.half_carry = (a & 0x0F) < (b & 0x0F) + carry;
        self.registers.flags.negative = true;
        self.registers.flags.zero = result & 0xFF == 0x00;
        self.registers.flags.carry = (a as u16) < b as u16 + carry as u16;

        self.registers.a = result as u8;
    }

    fn sbc_a_a(&mut self) {
        let a = self.registers.a;
        self.sbc(a);
    }

    fn sbc_a_b(&mut self) {
        let b = self.registers.b;
        self.sbc(b);
    }

    fn sbc_a_c(&mut self) {
        let c = self.registers.c;
        self.sbc(c);
    }

    fn sbc_a_d(&mut self) {
        let d = self.registers.d;
        self.sbc(d);
    }

    fn sbc_a_e(&mut self) {
        let e = self.registers.e;
        self.sbc(e);
    }

    fn sbc_a_h(&mut self) {
        let h = self.registers.h;
        self.sbc(h);
    }

    fn sbc_a_l(&mut self) {
        let l = self.registers.l;
        self.sbc(l);
    }

    fn sbc_a_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        self.sbc(val);
    }

    fn sbc_a_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.sbc(val);
    }

    fn scf(&mut self) {
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = true;
    }

    fn set(&mut self, mut b: u8, n: u8) -> u8 {
        let shift = 0x01 << n;
        b |= shift;

        b
    }

    fn set_0_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.set(a, 0x00);
    }

    fn set_0_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.set(b, 0x00);
    }

    fn set_0_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.set(c, 0x00);
    }

    fn set_0_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.set(d, 0x00);
    }

    fn set_0_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.set(e, 0x00);
    }

    fn set_0_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.set(h, 0x00);
    }

    fn set_0_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.set(val, 0x00));
    }

    fn set_0_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.set(l, 0x00);
    }

    fn set_1_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.set(a, 0x01);
    }

    fn set_1_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.set(b, 0x01);
    }

    fn set_1_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.set(c, 0x01);
    }

    fn set_1_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.set(d, 0x01);
    }

    fn set_1_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.set(e, 0x01);
    }

    fn set_1_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.set(h, 0x01);
    }

    fn set_1_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.set(val, 0x01));
    }

    fn set_1_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.set(l, 0x01);
    }

    fn set_2_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.set(a, 0x02);
    }

    fn set_2_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.set(b, 0x02);
    }

    fn set_2_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.set(c, 0x02);
    }

    fn set_2_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.set(d, 0x02);
    }

    fn set_2_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.set(e, 0x02);
    }

    fn set_2_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.set(h, 0x02);
    }

    fn set_2_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.set(val, 0x02));
    }

    fn set_2_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.set(l, 0x02);
    }

    fn set_3_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.set(a, 0x03);
    }

    fn set_3_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.set(b, 0x03);
    }

    fn set_3_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.set(c, 0x03);
    }

    fn set_3_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.set(d, 0x03);
    }

    fn set_3_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.set(e, 0x03);
    }

    fn set_3_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.set(h, 0x03);
    }

    fn set_3_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.set(val, 0x03));
    }

    fn set_3_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.set(l, 0x03);
    }

    fn set_4_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.set(a, 0x04);
    }

    fn set_4_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.set(b, 0x04);
    }

    fn set_4_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.set(c, 0x04);
    }

    fn set_4_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.set(d, 0x04);
    }

    fn set_4_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.set(e, 0x04);
    }

    fn set_4_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.set(h, 0x04);
    }

    fn set_4_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.set(val, 0x04));
    }

    fn set_4_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.set(l, 0x04);
    }

    fn set_5_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.set(a, 0x05);
    }

    fn set_5_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.set(b, 0x05);
    }

    fn set_5_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.set(c, 0x05);
    }

    fn set_5_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.set(d, 0x05);
    }

    fn set_5_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.set(e, 0x05);
    }

    fn set_5_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.set(h, 0x05);
    }

    fn set_5_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.set(val, 0x05));
    }

    fn set_5_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.set(l, 0x05);
    }

    fn set_6_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.set(a, 0x06);
    }

    fn set_6_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.set(b, 0x06);
    }

    fn set_6_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.set(c, 0x06);
    }

    fn set_6_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.set(d, 0x06);
    }

    fn set_6_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.set(e, 0x06);
    }

    fn set_6_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.set(h, 0x06);
    }

    fn set_6_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.set(val, 0x06));
    }

    fn set_6_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.set(l, 0x06);
    }

    fn set_7_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.set(a, 0x07);
    }

    fn set_7_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.set(b, 0x07);
    }

    fn set_7_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.set(c, 0x07);
    }

    fn set_7_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.set(d, 0x07);
    }

    fn set_7_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.set(e, 0x07);
    }

    fn set_7_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.set(h, 0x07);
    }

    fn set_7_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.set(val, 0x07));
    }

    fn set_7_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.set(l, 0x07);
    }

    fn sla(&mut self, mut b: u8) -> u8 {
        let carry = b & 0x80 == 0x80;
        b = b << 0x01;

        self.registers.flags.zero = b == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = carry;

        b
    }

    fn sla_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.sla(a);
    }

    fn sla_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.sla(b);
    }

    fn sla_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.sla(c);
    }

    fn sla_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.sla(d);
    }

    fn sla_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.sla(e);
    }

    fn sla_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.sla(h);
    }

    fn sla_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.sla(val));
    }

    fn sla_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.sla(l);
    }

    fn sra(&mut self, mut b: u8) -> u8 {
        let carry = if b & 0x01 == 0x01 { true } else { false };
        b = ((b as i8) >> 0x01) as u8; // This cast preserves the sign bit for signed numbers

        self.registers.flags.zero = b == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = carry;

        b
    }

    fn sra_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.sra(a);
    }

    fn sra_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.sra(b);
    }

    fn sra_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.sra(c);
    }

    fn sra_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.sra(d);
    }

    fn sra_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.sra(e);
    }

    fn sra_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.sra(h);
    }

    fn sra_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.sra(val));
    }

    fn sra_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.sra(l);
    }

    fn srl(&mut self, mut b: u8) -> u8 {
        let carry = b & 0x01 == 0x01;
        b = b >> 0x01;

        self.registers.flags.zero = b == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = carry;

        b
    }

    fn srl_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.srl(a);
    }

    fn srl_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.srl(b);
    }

    fn srl_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.srl(c);
    }

    fn srl_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.srl(d);
    }

    fn srl_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.srl(e);
    }

    fn srl_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.srl(h);
    }

    fn srl_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.srl(val));
    }

    fn srl_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.srl(l);
    }

    fn sub(&mut self, b: u8) {
        let r = self.registers.a.wrapping_sub(b);

        self.registers.flags.zero = r == 0x00;
        self.registers.flags.negative = true;
        self.registers.flags.half_carry = (self.registers.a & 0x0F) < (b & 0x0F);
        self.registers.flags.carry = self.registers.a < b;

        self.registers.a = r;
    }

    fn sub_a(&mut self) {
        let a = self.registers.a;
        self.sub(a);
    }

    fn sub_b(&mut self) {
        let b = self.registers.b;
        self.sub(b);
    }

    fn sub_c(&mut self) {
        let c = self.registers.c;
        self.sub(c);
    }

    fn sub_d(&mut self) {
        let d = self.registers.d;
        self.sub(d);
    }

    fn sub_e(&mut self) {
        let e = self.registers.e;
        self.sub(e);
    }

    fn sub_h(&mut self) {
        let h = self.registers.h;
        self.sub(h);
    }

    fn sub_l(&mut self) {
        let l = self.registers.l;
        self.sub(l);
    }

    fn sub_hl(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        self.sub(val);
    }

    fn sub_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.sub(val);
    }

    fn swap(&mut self, b: u8) -> u8 {
        let result = (b >> 0x04) | (b << 0x04);

        self.registers.flags.zero = result == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = false;

        result
    }

    fn swap_a(&mut self) {
        let a = self.registers.a;
        self.registers.a = self.swap(a);
    }

    fn swap_b(&mut self) {
        let b = self.registers.b;
        self.registers.b = self.swap(b);
    }

    fn swap_c(&mut self) {
        let c = self.registers.c;
        self.registers.c = self.swap(c);
    }

    fn swap_d(&mut self) {
        let d = self.registers.d;
        self.registers.d = self.swap(d);
    }

    fn swap_e(&mut self) {
        let e = self.registers.e;
        self.registers.e = self.swap(e);
    }

    fn swap_h(&mut self) {
        let h = self.registers.h;
        self.registers.h = self.swap(h);
    }

    fn swap_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        interconnect.write_u8(self.registers.get_hl(), self.swap(val));
    }

    fn swap_l(&mut self) {
        let l = self.registers.l;
        self.registers.l = self.swap(l);
    }

    fn xor(&mut self, b: u8) {
        self.registers.a ^= b;
        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = false;
    }

    fn xor_a(&mut self) {
        let a = self.registers.a;
        self.xor(a);
    }

    fn xor_b(&mut self) {
        let b = self.registers.b;
        self.xor(b);
    }

    fn xor_c(&mut self) {
        let c = self.registers.c;
        self.xor(c);
    }

    fn xor_d(&mut self) {
        let d = self.registers.d;
        self.xor(d);
    }

    fn xor_e(&mut self) {
        let e = self.registers.e;
        self.xor(e);
    }

    fn xor_h(&mut self) {
        let h = self.registers.h;
        self.xor(h);
    }

    fn xor_hl_ptr(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        self.xor(val);
    }

    fn xor_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.xor(val);
    }

    fn xor_l(&mut self) {
        let l = self.registers.l;
        self.xor(l);
    }

    fn relative_jump(&mut self, offset: u8) {
        // If the sign bit is there, negate the PC by the difference
        // between 256 and the offset
        if offset & 0x80 == 0x80 {
            self.registers.pc -= 0x100 - offset as u16;
        } else {
            self.registers.pc += offset as u16;
        }
    }
}
