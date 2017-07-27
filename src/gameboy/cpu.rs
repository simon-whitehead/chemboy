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

    pub fn reset(&mut self) {
        self.registers.pc = 0x100;
    }

    fn get_operand_from_opcode(&self, interconnect: &Interconnect, opcode: &OpCode) -> Operand {
        let operand_start = self.registers.pc + 0x01;

        match opcode.argument_type {
            ArgumentType::Implied => Operand::None,
            ArgumentType::Imm8 => Operand::Imm8(interconnect.read_byte(operand_start)),
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
        let byte = interconnect.read_byte(self.registers.pc);
        println!("Read 0x{:02X} from 0x{:04X}", byte, self.registers.pc);

        if let Some(opcode) = OpCode::from_byte(byte) {
            let operand = self.get_operand_from_opcode(interconnect, &opcode);

            self.registers.pc += opcode.length - 1;

            match (opcode.mnemonic, opcode.argument_type) {
                ("DEC B", ArgumentType::Implied) => self.dec_b(),
                ("LD B, {imm8}", ArgumentType::Imm8) => self.ld_b_imm8(&operand),
                ("LD C, {imm8}", ArgumentType::Imm8) => self.ld_c_imm8(&operand),
                ("JR NZ, {imm8}", ArgumentType::Imm8) => self.jr_nz_imm8(&operand),
                ("LD HL, {imm16}", ArgumentType::Imm16) => self.ld_hl_imm16(&operand),
                ("LD (HLD), A", ArgumentType::Implied) => self.ld_hld_a(interconnect),
                ("JP {imm16}", ArgumentType::Imm16) => self.jp_imm16(&operand),
                ("XOR A", ArgumentType::Implied) => self.xor_a(),
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
        let b = &mut self.registers.b;
        *b = b.wrapping_sub(0x01);

        self.registers.flags.zero = *b == 0x00;
        self.registers.flags.n = true;
        self.registers.flags.h = (*b & 0x0F) == 0x0F;
    }

    fn ld_b_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.registers.b = val;
    }

    fn ld_c_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        self.registers.c = val;
    }

    fn jr_nz_imm8(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm8();
        println!("Jumping back {} bytes", val as i8);
        self.relative_jump(val);
    }

    fn ld_hl_imm16(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm16();
        self.registers.set_hl(val);
    }

    fn ld_hld_a(&mut self, interconnect: &mut Interconnect) {
        let addr = self.registers.get_hl();
        interconnect.write_byte(addr, self.registers.a);
        self.registers.set_hl(addr - 0x01);
    }

    fn jp_imm16(&mut self, operand: &Operand) {
        let addr = operand.unwrap_imm16();
        self.registers.set_pc(addr);
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
