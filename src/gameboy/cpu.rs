// Simon Whitehead, 2016

use gameboy::registers;
use gameboy::{MAX_CPU_CYCLES, Interconnect, Interrupt};
use gameboy::opcode::{OpCode, Operand, ArgumentType};

pub struct Cpu {
    pub registers: registers::Registers,
}

impl Cpu {
    pub fn new(gameboy_color: bool) -> Cpu {
        Cpu { registers: registers::Registers::new(gameboy_color) }
    }

    pub fn reset(&mut self, interconnect: &mut Interconnect) {
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

        while cycles < MAX_CPU_CYCLES {
            let c = self.step(interconnect)?;
            cycles += c as usize;
            interconnect.step(cycles)?;
            cycles += self.handle_interrupts(interconnect) as usize;
        }

        Ok(())
    }

    pub fn handle_interrupts(&mut self, interconnect: &mut Interconnect) -> u8 {
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

        0x00
    }

    pub fn step(&mut self, interconnect: &mut Interconnect) -> Result<u8, String> {
        let byte = interconnect.read_u8(self.registers.pc);

        if let Some(opcode) = OpCode::from_byte(byte, false) {
            let mut cycles = opcode.cycles;
            let operand = self.get_operand_from_opcode(interconnect, &opcode);

            // println!("Read 0x{:02X} from 0x{:04X}", byte, self.registers.pc);
            self.registers.pc += opcode.length;

            match opcode.code {
                0x00 => (),
                0x01 => self.ld_bc_imm16(&operand),
                0x02 => self.ld_bc_a(interconnect),
                0x03 => self.inc_bc(),
                0x05 => self.dec_b(),
                0x06 => self.ld_b_imm8(&operand),
                0x09 => self.add_hl_bc(),
                0x0A => self.ld_a_bc(interconnect),
                0x0B => self.dec_bc(),
                0x0C => self.inc_c(),
                0x0D => self.dec_c(),
                0x0E => self.ld_c_imm8(&operand),
                0x11 => self.ld_de_imm16(&operand),
                0x12 => self.ld_de_a(interconnect),
                0x13 => self.inc_de(),
                0x16 => self.ld_d_imm8(&operand),
                0x18 => self.jp_imm8(&operand),
                0x19 => self.add_hl_de(),
                0x1A => self.ld_a_de(interconnect),
                0x1C => self.inc_e(),
                0x20 => self.jr_nz_imm8(&operand),
                0x21 => self.ld_hl_imm16(&operand),
                0x22 => self.ld_hli_a(interconnect),
                0x23 => self.inc_hl(),
                0x28 => self.jr_z_imm8(&operand),
                0x2A => self.ld_a_hli(interconnect),
                0x2C => self.inc_l(),
                0x2F => self.cpl(),
                0x31 => self.ld_sp_imm16(&operand),
                0x32 => self.ld_hld_a(interconnect),
                0x34 => self.inc_hl_ptr(interconnect),
                0x35 => self.dec_hl(interconnect),
                0x36 => self.ld_hl_imm8(&operand, interconnect),
                0x3C => self.inc_a(),
                0x3D => self.dec_a(),
                0x3E => self.ld_a_imm8(&operand),
                0x46 => self.ld_b_hl(interconnect),
                0x47 => self.ld_b_a(),
                0x4E => self.ld_c_hl(interconnect),
                0x4F => self.ld_c_a(),
                0x54 => self.ld_d_h(),
                0x56 => self.ld_d_hl(interconnect),
                0x57 => self.ld_d_a(),
                0x5E => self.ld_e_hl(interconnect),
                0x5F => self.ld_e_a(),
                0x60 => self.ld_h_b(),
                0x68 => self.ld_l_b(),
                0x69 => self.ld_l_c(),
                0x6E => self.ld_l_hl(interconnect),
                0x6F => self.ld_l_a(),
                0x77 => self.ld_hl_a(interconnect),
                0x78 => self.ld_a_b(),
                0x79 => self.ld_a_c(),
                0x7C => self.ld_a_h(),
                0x7E => self.ld_a_hl(interconnect),
                0x85 => self.add_a_l(),
                0x87 => self.add_a_a(),
                0x9A => self.sbc_a_d(),
                0xA1 => self.and_c(),
                0xA7 => self.and_a(),
                0xA8 => self.xor_b(),
                0xA9 => self.xor_c(),
                0xAF => self.xor_a(),
                0xB0 => self.or_b(),
                0xB1 => self.or_c(),
                0xC0 => self.ret_nz(interconnect),
                0xC1 => self.pop_bc(interconnect),
                0xC2 => self.jp_nz_imm16(&operand),
                0xC3 => self.jp_imm16(&operand),
                0xC5 => self.push_bc(interconnect),
                0xC8 => self.ret_z(interconnect),
                0xC9 => self.ret(interconnect),
                0xCA => self.jp_z_imm16(&operand),
                0xCB => {
                    cycles = self.handle_extended_opcode(interconnect);
                }
                0xCD => self.call(operand.unwrap_imm16(), interconnect),
                0xD1 => self.pop_de(interconnect),
                0xD5 => self.push_de(interconnect),
                0xD9 => self.reti(interconnect),
                0xE0 => self.ld_ff00_imm8_a(&operand, interconnect),
                0xE1 => self.pop_hl(interconnect),
                0xE2 => self.ld_ff00_c_a(interconnect),
                0xE5 => self.push_hl(interconnect),
                0xE6 => self.and_imm8(&operand),
                0xE9 => self.jp_hl(),
                0xEA => self.ld_imm16_a(&operand, interconnect),
                0xEF => self.call(0x28, interconnect),
                0xF0 => self.ld_a_ff00_imm8(&operand, interconnect),
                0xF1 => self.pop_af(interconnect),
                0xF3 => self.di(interconnect),
                0xF5 => self.push_af(interconnect),
                0xFA => self.ld_a_imm16(&operand, interconnect),
                0xFB => self.ei(interconnect),
                0xFE => self.cp_n(&operand),
                _ => {
                    return Err(format!("Could not match opcode: {:02X} at offset: {:04X}",
                                       opcode.code,
                                       self.registers.pc))
                }
            }

            return Ok(cycles);
        }

        Err(format!("Unknown opcode: 0x{:02X} at offset: 0x{:04X}",
                    byte,
                    self.registers.pc))
    }

    pub fn handle_extended_opcode(&mut self, interconnect: &mut Interconnect) -> u8 {
        let byte = interconnect.read_u8(self.registers.pc);

        if let Some(opcode) = OpCode::from_byte(byte, true) {
            let operand = self.get_operand_from_opcode(interconnect, &opcode);

            // println!("Read 0x{:02X} from 0x{:04X}", byte, self.registers.pc);
            self.registers.pc += opcode.length;

            match opcode.code {
                0x27 => self.sla_a(),
                0x37 => self.swap_a(),
                0x87 => self.res_0_a(),
                0xFE => self.set_7_hl(interconnect),
                _ => {
                    panic!("Could not match opcode: {:02X} at offset: {:04X}",
                           opcode.code,
                           self.registers.pc)
                }
            }

            return opcode.cycles + 0x01;
        }

        panic!("Unknown extended opcode: 0x{:02X} at offset: 0x{:04X}",
               byte,
               self.registers.pc);
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
        self.registers.flags.half_carry = ((hl & 0x0FFF) + (bc & 0x0FFF)) & 0x1000 == 0x1000;
        self.registers.flags.negative = false;
        self.registers.flags.carry = r > 0xFFFF;
    }

    fn add_hl_de(&mut self) {
        let hl = self.registers.get_hl();
        let de = self.registers.get_de();

        let r = hl.wrapping_add(de);

        self.registers.set_hl(r);
        self.registers.flags.half_carry = ((hl & 0x0FFF) + (de & 0x0FFF)) & 0x1000 == 0x1000;
        self.registers.flags.negative = false;
        self.registers.flags.carry = r > 0xFFFF;
    }

    fn and_a(&mut self) {
        let r = self.registers.a & self.registers.a;

        self.registers.a = r;
        self.registers.flags.zero = r == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = true;
        self.registers.flags.carry = false;
    }

    fn and_c(&mut self) {
        let r = self.registers.a & self.registers.c;

        self.registers.a = r;
        self.registers.flags.zero = r == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = true;
        self.registers.flags.carry = false;
    }

    fn and_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        let r = self.registers.a & val;

        self.registers.a = r;
        self.registers.flags.zero = r == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = true;
        self.registers.flags.carry = false;
    }

    fn call(&mut self, addr: u16, interconnect: &mut Interconnect) {
        self.registers.sp -= 0x02;
        interconnect.write_u16(self.registers.sp as u16, self.registers.pc);
        self.registers.pc = addr;
    }

    fn cpl(&mut self) {
        self.registers.a = !self.registers.a as u8;
        self.registers.flags.negative = true;
        self.registers.flags.half_carry = true;
    }

    fn cp_n(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        let r = self.registers.a;
        let result = r.wrapping_sub(val);

        self.registers.flags.zero = result == 0x00;
        self.registers.flags.negative = true;
        self.registers.flags.half_carry = (r & 0x0F) == 0x00;
        self.registers.flags.carry = self.registers.a < val;
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

    fn dec_c(&mut self) {
        let r = self.registers.c;
        self.registers.c = r.wrapping_sub(0x01);

        self.registers.flags.zero = self.registers.c == 0x00;
        self.registers.flags.negative = true;
        self.registers.flags.half_carry = (r & 0x0F) == 0x00;
    }

    fn dec_hl(&mut self, interconnect: &mut Interconnect) {
        let val = interconnect.read_u8(self.registers.get_hl());
        let result = val.wrapping_sub(0x01);
        interconnect.write_u8(self.registers.get_hl(), result);

        self.registers.flags.zero = result == 0x00;
        self.registers.flags.negative = true;
        self.registers.flags.half_carry = (val & 0x0F) == 0x00;
    }

    fn di(&mut self, interconnect: &mut Interconnect) {
        interconnect.irq.enabled = false;
    }

    fn ei(&mut self, interconnect: &mut Interconnect) {
        interconnect.irq.enabled = true;
    }

    fn inc_a(&mut self) {
        let r = self.registers.a;
        self.registers.a = r.wrapping_add(0x01);

        self.registers.flags.zero = self.registers.a == 0x00;
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
        self.registers.flags.half_carry = (val & 0x0F) == 0x00;
    }

    fn inc_l(&mut self) {
        let r = self.registers.l;
        self.registers.l = r.wrapping_add(0x01);

        self.registers.flags.zero = self.registers.l == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = (r & 0x0F) + 0x01 > 0x0F;
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

    fn ld_a_de(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_de();
        let val = interconnect.read_u8(addr);
        self.registers.a = val;
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

    fn ld_b_a(&mut self) {
        self.registers.b = self.registers.a;
    }

    fn ld_b_hl(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        let val = interconnect.read_u8(addr);
        self.registers.b = val;
    }

    fn ld_c_a(&mut self) {
        self.registers.c = self.registers.a;
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

    fn ld_c_hl(&mut self, interconnect: &mut Interconnect) {
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

    fn ld_d_h(&mut self) {
        self.registers.d = self.registers.h;
    }

    fn ld_d_hl(&mut self, interconnect: &mut Interconnect) {
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

    fn ld_e_hl(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        let val = interconnect.read_u8(addr);
        self.registers.e = val;
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

    fn ld_h_b(&mut self) {
        self.registers.h = self.registers.b;
    }

    fn ld_hl_a(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        interconnect.write_u8(addr, self.registers.a);
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

    fn ld_l_a(&mut self) {
        self.registers.l = self.registers.a;
    }


    fn ld_l_b(&mut self) {
        self.registers.l = self.registers.b;
    }

    fn ld_l_c(&mut self) {
        self.registers.l = self.registers.c;
    }

    fn ld_l_hl(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        let val = interconnect.read_u8(addr);
        self.registers.l = val;
    }

    fn ld_sp_imm16(&mut self, operand: &Operand) {
        let addr = operand.unwrap_imm16();
        self.registers.sp = addr as usize;
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

    fn res_0_a(&mut self) {
        self.registers.a &= !0x01;
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

    fn sbc_a_d(&mut self) {
        let carry = if self.registers.flags.carry {
            0x01
        } else {
            0x00
        };

        let result = self.registers
            .a
            .wrapping_sub(self.registers.d)
            .wrapping_sub(carry);

        self.registers.flags.half_carry = (self.registers.a & 0x0F) <
                                          (self.registers.d & 0x0F) + carry;
        self.registers.flags.negative = true;
        self.registers.flags.zero = result & 0xFF == 0x00;
        self.registers.flags.carry = self.registers.a & 0x0F < (self.registers.d + carry);

        self.registers.a = result as u8;
    }

    fn set_7_hl(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        let val = interconnect.read_u8(addr);

        interconnect.write_u8(addr, val | 0x80);
    }

    fn sla_a(&mut self) {
        let carry = self.registers.a & 0x80 == 0x80;
        self.registers.a = self.registers.a << 0x01;

        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = carry;
    }

    fn swap_a(&mut self) {
        let result = (self.registers.a >> 4) | (self.registers.a << 4);

        self.registers.a = result;
        self.registers.flags.zero = result == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = false;
    }

    fn xor_a(&mut self) {
        self.registers.a ^= self.registers.a;
        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = false;
    }

    fn xor_b(&mut self) {
        self.registers.a ^= self.registers.b;
        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = false;
    }

    fn xor_c(&mut self) {
        self.registers.a ^= self.registers.c;
        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.negative = false;
        self.registers.flags.half_carry = false;
        self.registers.flags.carry = false;
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
