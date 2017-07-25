use ::gameboy::registers;

pub enum Operand {
    None,
    Imm8(u8),
    Imm16(u16),
    Register(registers::Registers),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArgumentType {
    None,
    Imm8,
    Imm16,
    Register,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpCode {
    pub code: u8,
    pub mnemonic: &'static str,
    pub length: u8,
    pub time: u8,
    pub argument_type: ArgumentType,
}

static OpCodes: [OpCode; 4] = [OpCode {
                                   code: 0x31,
                                   mnemonic: "LD SP, {{0}}",
                                   length: 3,
                                   time: 3,
                                   argument_type: ArgumentType::Imm16,
                               },
                               OpCode {
                                   code: 0x21,
                                   mnemonic: "LD HL, {{0}}",
                                   length: 3,
                                   time: 3,
                                   argument_type: ArgumentType::Imm16,
                               },
                               OpCode {
                                   code: 0x32,
                                   mnemonic: "LD (HLD), A",
                                   length: 1,
                                   time: 2,
                                   argument_type: ArgumentType::None,
                               },
                               OpCode {
                                   code: 0xAF,
                                   mnemonic: "XOR A",
                                   length: 1,
                                   time: 1,
                                   argument_type: ArgumentType::None,
                               }];