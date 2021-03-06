
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
            let mut line = String::from(format!("{:04X}: ", idx));
            let mnemonic = opcode.mnemonic;
            let formatted_opcode = if opcode.extended {
                format!("CB {:02X}", opcode.code)
            } else {
                format!("{:02X}", opcode.code)
            };
            match opcode.argument_type {
                ArgumentType::Implied => {
                    let v = format!("{}", formatted_opcode);
                    line.push_str(&format!("{:<8} {}", v, mnemonic)[..]);
                    result.push(line);
                    idx += 0x01;
                }
                ArgumentType::Imm8 => {
                    let imm8 = bytecode[idx + 0x01];
                    let v = format!("{} {:02X}", formatted_opcode, imm8);
                    line.push_str(&format!("{:<8} {}",
                                           v,
                                           mnemonic.replace("{imm8}", &format!("${:02X}", imm8))));
                    result.push(line);
                    idx += 0x02;
                }
                ArgumentType::Imm16 => {
                    let imm16 = LittleEndian::read_u16(&bytecode[idx + 0x01..]);
                    let v = format!("{} {:02X} {:02X}",
                                    formatted_opcode,
                                    (imm16 & 0xFF) as u8,
                                    (imm16 >> 0x08) as u8);
                    line.push_str(&format!("{:<8} {}",
                                           v,
                                           mnemonic.replace("{imm16}", &format!("${:04X}", imm16)))
                                           );
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