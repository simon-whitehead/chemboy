// Simon Whitehead, 2016

use byteorder::{ByteOrder, LittleEndian};

use gameboy::registers;
use gameboy::Interconnect;
use gameboy::memory_map;
use gameboy::opcode::{OpCode, Operand, ArgumentType};

const MAX_CYCLES: usize = 69905;

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

    pub fn cycle(&mut self, interconnect: &mut Interconnect) {
        let mut cycles = 0;

        while cycles < MAX_CYCLES {
            let c = self.step(interconnect);
            cycles += c as usize;
        }
    }

    pub fn step(&mut self, interconnect: &mut Interconnect) -> u8 {
        let byte = interconnect.read_u8(self.registers.pc);

        if let Some(opcode) = OpCode::from_byte(byte) {
            let operand = self.get_operand_from_opcode(interconnect, &opcode);

            println!("Read 0x{:02X} from 0x{:04X}", byte, self.registers.pc);
            self.registers.pc += opcode.length;

            match opcode.code {
                0x00 => (),
                0x05 => self.dec_b(),
                0x06 => self.ld_b_imm8(&operand),
                0x0D => self.dec_c(),
                0x0E => self.ld_c_imm8(&operand),
                0x20 => self.jr_nz_imm8(&operand, interconnect),
                0x21 => self.ld_hl_imm16(&operand),
                0x32 => self.ld_hld_a(interconnect),
                0x3E => self.ld_a_imm8(&operand),
                0xAF => self.xor_a(),
                0xC3 => self.jp_imm16(&operand),
                0xE0 => self.ld_ff00_imm8_a(&operand, interconnect),
                0xF3 => self.di(),
                _ => {
                    panic!("Could not match opcode mnemonic: 0x{:02X} at offset: 0x{:04X}",
                           opcode.code,
                           self.registers.pc)
                }
            }

            return opcode.cycles;
        }

        panic!("Unknown opcode: 0x{:02X} at offset: 0x{:04X}",
               byte,
               self.registers.pc);
    }

    fn dec_b(&mut self) {
        self.registers.b = self.registers.b.wrapping_sub(0x01);

        self.registers.flags.zero = self.registers.b == 0x00;
        self.registers.flags.n = true;
        self.registers.flags.h = (self.registers.b & 0x0F) == 0x0F;
    }

    fn dec_c(&mut self) {
        self.registers.c = self.registers.c.wrapping_sub(0x01);

        self.registers.flags.zero = self.registers.c == 0x00;
        self.registers.flags.n = true;
        self.registers.flags.h = (self.registers.c & 0x0F) == 0x0F;
    }

    fn di(&mut self) {
        self.registers.flags.ime = false;
    }

    fn ld_a_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.registers.a = val;
    }

    fn ld_b_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.registers.b = val;
    }

    fn ld_c_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.registers.c = val;
    }

    fn ld_ff00_imm8_a(&mut self, operand: &Operand, interconnect: &mut Interconnect) {
        let offset = operand.unwrap_imm8();
        let addr = 0xFF00 as u16 + offset as u16;
        interconnect.write_u8(addr, self.registers.a);
        println!("Wrote: {:02X} to {:04X}", self.registers.a, addr);
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

    fn jp_imm16(&mut self, operand: &Operand) {
        let addr = operand.unwrap_imm16();
        self.registers.set_pc(addr);
    }

    fn jr_nz_imm8(&mut self, operand: &Operand, interconnect: &Interconnect) {
        let offset = operand.unwrap_imm8();

        if self.registers.flags.zero == false {
            self.relative_jump(offset);
        }
    }

    fn xor_a(&mut self) {
        self.registers.a ^= self.registers.a;
        self.registers.flags.zero = self.registers.a == 0x00;
        self.registers.flags.n = false;
        self.registers.flags.h = false;
        self.registers.flags.cy = false;
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
