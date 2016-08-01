// Simon Whitehead, 2016

use byteorder::{ByteOrder, LittleEndian};

use gameboy::registers;
use gameboy::Interconnect;

pub struct Cpu {
    rom: Vec<u8>,
    pub registers: registers::Registers,
    pub interconnect: Interconnect,
}

impl Cpu {
    pub fn new(gameboy_color: bool, rom: Vec<u8>) -> Cpu {
        Cpu {
            rom: rom,
            registers: registers::Registers::new(gameboy_color),
            interconnect: Interconnect::new(),
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

                // Increment program counter
                self.registers.pc += 0x03;
            }
            0b00100001 => {
                // LD HL, <u16>
                let val = LittleEndian::read_u16(&self.rom[self.registers.pc..]);
                self.registers.l = (val & 0xFF) as u8;
                self.registers.h = ((val >> 8) & 0xFF) as u8;

                // Increment program counter
                self.registers.pc += 0x03;
            }
            0b00110010 => {
                // LD (HL-),A

            }
            0b10101111 => {
                // XOR A
                self.registers.a = self.registers.a ^ self.registers.a;
                self.registers.flags.zero = self.registers.a == 0;

                // Increment program counter
                self.registers.pc += 0x01;
            }
            _ => panic!("Unknown opcode: {:#X}", opcode),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn ld_sp_u16_should_set_stack_pointer() {
        let ldspu16 = vec![0x31, 0xFE, 0xFF];
        let mut cpu = Cpu::new(true, ldspu16);
        cpu.step();

        // SP == 0xFFFE
        assert_eq!(0xFFFE, cpu.registers.sp);
    }

    #[test]
    pub fn xor_a_zeros_a_and_sets_zero_flag() {
        let xor_a = vec![0xAF];
        let mut cpu = Cpu::new(true, xor_a);
        cpu.step();

        // A == 0, Z == 1
        assert_eq!(0, cpu.registers.a);
        assert_eq!(true, cpu.registers.flags.zero);
    }

    #[test]
    pub fn ld_hl_u16_should_split_between_h_and_l_registers() {
        let ldhlu16 = vec![0x21, 0xFF, 0x9F];
        let mut cpu = Cpu::new(true, ldhlu16);
        cpu.step();

        // H == 0x9F, L == 0xFF
        assert_eq!(0x9F, cpu.registers.h);
        assert_eq!(0xFF, cpu.registers.l);
    }
}
