use ::gameboy::registers;

pub enum Operand {
    None,
    Imm8(u8),
    Imm16(u16),
    Register(registers::Registers),
    Addr8(u8),
    Addr16(u16),
}

impl Operand {
    pub fn unwrap_imm16(&self) -> u16 {
        match *self {
            Operand::Imm16(addr) => addr,
            _ => panic!("Attempted to unwrap imm16 operand when it is not of type Imm16"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArgumentType {
    Implied,
    Imm8,
    Imm16,
    Register,
    Address,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpCode {
    pub code: u8,
    pub mnemonic: &'static str,
    pub length: u8,
    pub time: u8,
    pub argument_type: ArgumentType,
}

impl OpCode {
    pub fn from_byte<'opcode>(byte: u8) -> Option<&'opcode OpCode> {
        OpCodes.iter().find(|opcode| opcode.code == byte)
    }

    pub fn from_mnemonic<S>(input: S) -> Option<OpCode>
        where S: Into<String>
    {
        let input = input.into();
        OpCodes.iter()
            .find(|opcode| opcode.mnemonic == input.to_uppercase())
            .cloned()
    }
}

static OpCodes: [OpCode; 5] = [OpCode {
                                   code: 0x31,
                                   mnemonic: "LD SP, {imm16}",
                                   length: 3,
                                   time: 3,
                                   argument_type: ArgumentType::Imm16,
                               },
                               OpCode {
                                   code: 0x21,
                                   mnemonic: "LD HL, {imm16}",
                                   length: 3,
                                   time: 3,
                                   argument_type: ArgumentType::Imm16,
                               },
                               OpCode {
                                   code: 0x32,
                                   mnemonic: "LD (HLD), A",
                                   length: 1,
                                   time: 2,
                                   argument_type: ArgumentType::Implied,
                               },
                               OpCode {
                                   code: 0xAF,
                                   mnemonic: "XOR A",
                                   length: 1,
                                   time: 1,
                                   argument_type: ArgumentType::Implied,
                               },
                               OpCode {
                                   code: 0xC3,
                                   mnemonic: "JP {imm16}",
                                   length: 3,
                                   time: 4,
                                   argument_type: ArgumentType::Imm16,
                               }];