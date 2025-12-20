
use crate::gameboy;
use gameboy::opcodes::OpCode;

pub struct DisassembledLine {
    opcode: OpCode,
}