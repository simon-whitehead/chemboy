
use std::io::Cursor;

use byteorder::{ByteOrder, LittleEndian};

use gameboy::registers;

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

    pub fn step(&mut self) {
        let opcode = self.rom[self.registers.pc];
        self.registers.pc += 1;

        match opcode {
            0b00110001 => {
                // LD SP, <u16>
                let val = LittleEndian::read_u16(&self.rom[self.registers.pc..]);
                self.registers.sp = val as usize;
                self.registers.pc += 0x03;
                println!("Stack pointer: {:#X}", self.registers.sp);
            }
            0b10101111 => {
                // XOR A
                self.registers.a = self.registers.a ^ self.registers.a;
            }
            _ => panic!("Unknown opcode: {:#X}", opcode),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn xor_a_zeros_a_and_sets_zero_flag() {
        let xor_a = vec![0xAF];
        let mut cpu = Cpu::new(true, xor_a);

        cpu.step();

        assert_eq!(true, cpu.registers.flags.zero);
        assert_eq!(0, cpu.registers.a);
    }
}
