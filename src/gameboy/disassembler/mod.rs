
mod disassembled_line;

pub use self::disassembled_line::DisassembledLine;

use byteorder::{ByteOrder, LittleEndian};

use gameboy::opcodes::{ArgumentType, OpCode};

pub fn disassemble(bytecode: &[u8]) -> Vec<String> {
    let mut result = Vec::new();
    let mut idx: usize = 0;
    let mut extended = false;
    while idx < bytecode.len() {
        let b = bytecode[idx];
        if b == 0xCB {
            extended = true;
            idx += 0x01;
            continue;
        }
        if let Some(opcode) = OpCode::from_byte(b, extended) {
            let mut line = String::new();
            let mnemonic = opcode.mnemonic;
            if opcode.extended {
                line.push_str("CB ");
            }
            match opcode.argument_type {
                ArgumentType::Implied => {
                    line.push_str(&format!("{:02X} {}", opcode.code, mnemonic)[..]);
                    result.push(line);
                    idx += 0x01;
                }
                ArgumentType::Imm8 => {
                    let imm8 = bytecode[idx + 0x01];
                    line.push_str(&format!("{:02X} {} ${:02X}",
                                           opcode.code,
                                           mnemonic.replace(" {imm8}", ""),
                                           imm8)[..]);
                    result.push(line);
                    idx += 0x02;
                }
                ArgumentType::Imm16 => {
                    let imm16 = LittleEndian::read_u16(&bytecode[idx + 0x01..]);
                    line.push_str(&format!("{:02X} {} ${:04X}",
                                           opcode.code,
                                           mnemonic.replace(" {imm16}", ""),
                                           imm16)[..]);
                    result.push(line);
                    idx += 0x03;
                }
                _ => (),
            }
        } else {
            idx += 0x01;
        }
        extended = false;
    }
    result
}