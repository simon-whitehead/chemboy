use gameboy::registers;

#[derive(Eq, PartialEq)]
pub enum Operand {
    None,
    Imm8(u8),
    Imm16(u16),
    Register(registers::Registers),
    Addr8(u8),
    Addr16(u16),
}

impl Operand {
    pub fn unwrap_imm8(&self) -> u8 {
        match *self {
            Operand::Imm8(val) => val,
            _ => panic!("Attempted to unwrap imm8 operand when it is not of type Imm8"),
        }
    }

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
    pub length: u16,
    pub cycles: u8,
    pub argument_type: ArgumentType,
    pub extended: bool,
}

impl OpCode {
    pub fn from_byte<'opcode>(byte: u8, extended: bool) -> Option<&'opcode OpCode> {
        if extended {
            EXT_OPCODES.iter().find(|opcode| opcode.code == byte)
        } else {
            OPCODES.iter().find(|opcode| opcode.code == byte)
        }
    }

    pub fn from_mnemonic<S>(input: S) -> Option<OpCode>
    where
        S: Into<String>,
    {
        let input = input.into();
        OPCODES
            .iter()
            .find(|opcode| opcode.mnemonic == input.to_uppercase())
            .cloned()
    }
}

static OPCODES: [OpCode; 50] = [
    OpCode {
        code: 0x00,
        mnemonic: "NOP",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x01,
        mnemonic: "LD BC, {imm16}",
        length: 3,
        cycles: 12,
        argument_type: ArgumentType::Imm16,
        extended: false,
    },
    OpCode {
        code: 0x05,
        mnemonic: "DEC B",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x06,
        mnemonic: "LD B, {imm8}",
        length: 2,
        cycles: 8,
        argument_type: ArgumentType::Imm8,
        extended: false,
    },
    OpCode {
        code: 0x0B,
        mnemonic: "DEC BC",
        length: 1,
        cycles: 8,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x0C,
        mnemonic: "INC C",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x0D,
        mnemonic: "DEC C",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x0E,
        mnemonic: "LD C, {imm8}",
        length: 2,
        cycles: 8,
        argument_type: ArgumentType::Imm8,
        extended: false,
    },
    OpCode {
        code: 0x11,
        mnemonic: "LD DE, {imm16}",
        length: 3,
        cycles: 12,
        argument_type: ArgumentType::Imm16,
        extended: false,
    },
    OpCode {
        code: 0x12,
        mnemonic: "LD (DE), A",
        length: 1,
        cycles: 8,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x16,
        mnemonic: "LD D, {imm8}",
        length: 2,
        cycles: 8,
        argument_type: ArgumentType::Imm8,
        extended: false,
    },
    OpCode {
        code: 0x19,
        mnemonic: "ADD HL, DE",
        length: 1,
        cycles: 8,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x20,
        mnemonic: "JR NZ, {imm8}",
        length: 2,
        cycles: 8,
        argument_type: ArgumentType::Imm8,
        extended: false,
    },
    OpCode {
        code: 0x21,
        mnemonic: "LD HL, {imm16}",
        length: 3,
        cycles: 12,
        argument_type: ArgumentType::Imm16,
        extended: false,
    },
    OpCode {
        code: 0x23,
        mnemonic: "INC HL",
        length: 1,
        cycles: 8,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x2A,
        mnemonic: "LD A, (HLI)",
        length: 1,
        cycles: 8,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x2F,
        mnemonic: "CPL",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x31,
        mnemonic: "LD SP, {imm16}",
        length: 3,
        cycles: 12,
        argument_type: ArgumentType::Imm16,
        extended: false,
    },
    OpCode {
        code: 0x32,
        mnemonic: "LD (HLD), A",
        length: 1,
        cycles: 8,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x36,
        mnemonic: "LD (HL), {imm8}",
        length: 2,
        cycles: 12,
        argument_type: ArgumentType::Imm8,
        extended: false,
    },
    OpCode {
        code: 0x3E,
        mnemonic: "LD A, {imm8}",
        length: 2,
        cycles: 8,
        argument_type: ArgumentType::Imm8,
        extended: false,
    },
    OpCode {
        code: 0x47,
        mnemonic: "LD B, A",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x4F,
        mnemonic: "LD C, A",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x56,
        mnemonic: "LD D, (HL)",
        length: 1,
        cycles: 8,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x5E,
        mnemonic: "LD E, (HL)",
        length: 1,
        cycles: 8,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x5F,
        mnemonic: "LD E, A",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x78,
        mnemonic: "LD A, B",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x79,
        mnemonic: "LD A, C",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0x87,
        mnemonic: "AND A, A",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0xA1,
        mnemonic: "AND C",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0xA9,
        mnemonic: "XOR C",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0xAF,
        mnemonic: "XOR A",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0xB0,
        mnemonic: "OR B",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0xB1,
        mnemonic: "OR C",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0xC3,
        mnemonic: "JP {imm16}",
        length: 3,
        cycles: 12,
        argument_type: ArgumentType::Imm16,
        extended: false,
    },
    OpCode {
        code: 0xC9,
        mnemonic: "RET",
        length: 1,
        cycles: 8,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0xCB,
        mnemonic: "",
        length: 1,
        cycles: 0,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0xCD,
        mnemonic: "CALL {imm16}",
        length: 3,
        cycles: 12,
        argument_type: ArgumentType::Imm16,
        extended: false,
    },
    OpCode {
        code: 0xD5,
        mnemonic: "PUSH DE",
        length: 1,
        cycles: 16,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0xE0,
        mnemonic: "LD ($FF00+{imm8}), A",
        length: 2,
        cycles: 12,
        argument_type: ArgumentType::Imm8,
        extended: false,
    },
    OpCode {
        code: 0xE1,
        mnemonic: "POP HL",
        length: 1,
        cycles: 12,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0xE2,
        mnemonic: "LD ($FF00+C), A",
        length: 1,
        cycles: 8,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0xE6,
        mnemonic: "AND {imm8}",
        length: 2,
        cycles: 8,
        argument_type: ArgumentType::Imm8,
        extended: false,
    },
    OpCode {
        code: 0xE9,
        mnemonic: "JP (HL)",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0xEA,
        mnemonic: "LD ({imm16}), A",
        length: 3,
        cycles: 16,
        argument_type: ArgumentType::Imm16,
        extended: false,
    },
    OpCode {
        code: 0xEF,
        mnemonic: "RST 0x28",
        length: 1,
        cycles: 32,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0xF0,
        mnemonic: "LD A, ($FF00+{imm8})",
        length: 2,
        cycles: 12,
        argument_type: ArgumentType::Imm8,
        extended: false,
    },
    OpCode {
        code: 0xF3,
        mnemonic: "DI",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0xFB,
        mnemonic: "EI",
        length: 1,
        cycles: 4,
        argument_type: ArgumentType::Implied,
        extended: false,
    },
    OpCode {
        code: 0xFE,
        mnemonic: "CP {imm8}",
        length: 2,
        cycles: 8,
        argument_type: ArgumentType::Imm8,
        extended: false,
    },
];

static EXT_OPCODES: [OpCode; 2] = [
    OpCode {
        code: 0x37,
        mnemonic: "SWAP A",
        length: 1,
        cycles: 8,
        argument_type: ArgumentType::Implied,
        extended: true,
    },
    OpCode {
        code: 0x87,
        mnemonic: "RES 0, A",
        length: 1,
        cycles: 8,
        argument_type: ArgumentType::Implied,
        extended: true,
    },
];
