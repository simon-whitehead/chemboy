// Simon Whitehead, 2016

use byteorder::{ByteOrder, LittleEndian};

use gameboy::registers;
use gameboy::Interconnect;
use gameboy::memory_map;
use gameboy::opcode;

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
            opcode::LD_SP_u16 => {
                let val = LittleEndian::read_u16(&self.rom[self.registers.pc..]);
                self.registers.sp = val as usize;

                self.registers.pc += 0x02;
            }
            opcode::LD_HL_u16 => {
                let val = LittleEndian::read_u16(&self.rom[self.registers.pc..]);
                self.registers.l = (val & 0xFF) as u8;
                self.registers.h = ((val >> 8) & 0xFF) as u8;

                self.registers.pc += 0x02;
            }
            opcode::LD_HLD_A => {
                let val = self.registers.get_hl();
                self.interconnect.write_byte(val, self.registers.a);
                self.registers.set_hl(val - 1);
            }
            opcode::XOR_A => {
                self.registers.a = self.registers.a ^ self.registers.a;
                self.registers.flags.zero = self.registers.a == 0;
            }
            opcode::BIT => {
                let destination = self.rom[self.registers.pc];
                match destination {
                    opcode::BIT_7_H => {
                        let r = self.registers.h & (1 << 7) == 0;
                        self.registers.flags.n = false;
                        self.registers.flags.h = true;
                        self.registers.flags.zero = r;
                    }
                    _ => panic!("Unknown BIT destination"),
                }
                self.registers.pc += 0x01;
            }
            opcode::JR_NZ => {
                // Jump, if not zero
                if self.registers.flags.zero == false {
                    let offset = self.rom[self.registers.pc] as i8;
                    if (offset < 0) {
                        // Fix the twos compliment: add 1 then invert it
                        self.registers.pc -= (-(offset + 0x01) as usize);
                    } else {
                        self.registers.pc += (offset as usize);
                    }
                } else {
                    self.registers.pc += 0x01;
                }
            }
            _ => {
                panic!("Unknown opcode: {:#X} at offset: {:#X}",
                       opcode,
                       self.registers.pc)
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use ::gameboy::opcode::*;

    #[test]
    pub fn ld_sp_u16_should_set_stack_pointer() {
        let ldspu16 = gb_asm![
            LD_SP_u16 0xFE 0xFF
        ];
        let mut cpu = Cpu::new(true, ldspu16);
        cpu.step();

        // SP == 0xFFFE
        assert_eq!(0xFFFE, cpu.registers.sp);
    }

    #[test]
    pub fn xor_a_zeros_a_and_sets_zero_flag() {
        let xor_a = gb_asm![XOR_A];
        let mut cpu = Cpu::new(true, xor_a);
        cpu.step();

        // A == 0, Z == 1
        assert_eq!(0, cpu.registers.a);
        assert_eq!(true, cpu.registers.flags.zero);
    }

    #[test]
    pub fn ld_hl_u16_should_split_between_h_and_l_registers() {
        let ldhlu16 = gb_asm![
            LD_HL_u16 0xFF 0x9F
        ];
        let mut cpu = Cpu::new(true, ldhlu16);
        cpu.step();

        // H == 0x9F, L == 0xFF
        assert_eq!(0x9F, cpu.registers.h);
        assert_eq!(0xFF, cpu.registers.l);
    }

    #[test]
    pub fn bit_7_h_sets_flags_correctly() {
        let bit7 = gb_asm![
            LD_HL_u16 0x00 0x80
            BIT BIT_7_H
        ];
        let mut cpu = Cpu::new(true, bit7);
        cpu.step();
        cpu.step();

        // Z == 0x00, H == 0x01, N == 0x00
        assert_eq!(false, cpu.registers.flags.zero);
        assert_eq!(true, cpu.registers.flags.h);
        assert_eq!(false, cpu.registers.flags.n);
    }

    #[test]
    pub fn jr_nz_does_not_jump_when_zero() {
        let bit7 = gb_asm![
            LD_HL_u16 0x00 0x80
            JR_NZ 0xFD
        ];
        let mut cpu = Cpu::new(true, bit7);
        cpu.registers.flags.zero = true;
        cpu.step();
        cpu.step();

        // Program counter should be 0x05
        assert_eq!(0x05, cpu.registers.pc);
    }

    #[test]
    pub fn jr_nz_does_jump_when_not_zero() {
        let bit7 = gb_asm![
            LD_HL_u16 0x00 0x80
            JR_NZ 0xFB
        ];
        let mut cpu = Cpu::new(true, bit7);
        cpu.registers.flags.zero = false;
        cpu.step();
        cpu.step();

        // Program counter should be 0x00
        assert_eq!(0x00, cpu.registers.pc);
    }
}
