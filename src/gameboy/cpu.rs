
use gameboy::registers;

pub struct Cpu {
    rom: Vec<u8>,
    pub registers: registers::Registers,
}

impl Cpu {
    pub fn new(gameboy_color: bool, rom: Vec<u8>) -> Cpu {
        println!("ROM length: {}", rom.len());
        Cpu {
            rom: rom,
            registers: registers::Registers::new(gameboy_color),
        }
    }
}
