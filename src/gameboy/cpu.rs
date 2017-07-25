// Simon Whitehead, 2016

use byteorder::{ByteOrder, LittleEndian};

use gameboy::registers;
use gameboy::Interconnect;
use gameboy::memory_map;
use gameboy::opcode;

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
            _ => {
                panic!("Unknown opcode: 0x{:02X} at offset: 0x{:04X}",
                       opcode,
                       self.registers.pc)
            }
        }
    }
}