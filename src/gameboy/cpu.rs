// Simon Whitehead, 2016

use byteorder::{ByteOrder, LittleEndian};

use gameboy::registers;
use gameboy::Interconnect;
use gameboy::memory_map;
use gameboy::opcode::{OpCode, Operand, ArgumentType};

pub struct Cpu {
    rom: Vec<u8>,
    pub registers: registers::Registers,
}

impl Cpu {
    pub fn new(gameboy_color: bool, rom: Vec<u8>) -> Cpu {
        Cpu {
            rom: rom,
            registers: registers::Registers::new(gameboy_color),
        }
    }

    fn get_operand_from_opcode(&self, interconnect: &Interconnect, opcode: &OpCode) -> Operand {
        match opcode.argument_type {
            ArgumentType::Implied => Operand::None,
            ArgumentType::Imm16 => {
                Operand::Imm16(LittleEndian::read_u16(&self.rom[self.registers.pc..]))
            }
            _ => panic!("Unknown opcode argument type"),
        }
    }

    pub fn step(&mut self, interconnect: &Interconnect) {
        let byte = self.rom[self.registers.pc];
        println!("Read 0x{:02X} from 0x{:04X}", byte, self.registers.pc);
        self.registers.pc += 1;

        if let Some(opcode) = OpCode::from_byte(byte) {
            let operand = self.get_operand_from_opcode(interconnect, &opcode);

            self.registers.pc += opcode.length as usize - 1;

            match (opcode.mnemonic, opcode.argument_type) {
                ("LD HL, {imm16}", ArgumentType::Imm16) => self.ld_hl_imm16(&operand),
                ("JP {imm16}", ArgumentType::Imm16) => self.jp_imm16(&operand),
                ("XOR A", ArgumentType::Implied) => self.xor_a(),
                _ => {
                    panic!("Unknown opcode: 0x{:02X} at offset: 0x{:04X}",
                           opcode.code,
                           self.registers.pc)
                }
            }
        } else {
            panic!("Unknown opcode: 0x{:02X} at offset: 0x{:04X}",
                   byte,
                   self.registers.pc)
        }
    }

    fn ld_hl_imm16(&mut self, operand: &Operand) {
        let val = operand.unwrap_imm16();
        self.registers.set_hl(val);
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
}