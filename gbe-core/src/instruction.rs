use crate::cpu::registers::{Register16, Register8};

pub enum Instruction {
    Nop,

    Adc(AddressingMode),
    AddA(AddressingMode),
    AddHL(Register16),
    AddSP,

    And(AddressingMode),
}

pub enum AddressingMode {
    Register(Register8),
    Indirect(Register16),
    Immediate8,
}

impl Instruction {

    pub fn decode(value: u8, prefixed: bool) -> Self {
        if prefixed {
            Self::prefixed(value)
        }
        else {
            Self::standard(value)
        }
    }

    fn standard(value: u8) -> Self {
        match value {
            0x00 => Self::Nop,

            0x09 => Self::AddHL(Register16::BC),
            0x19 => Self::AddHL(Register16::DE),
            0x29 => Self::AddHL(Register16::HL),
            0x39 => Self::AddHL(Register16::SP),

            0x80 => Self::AddA(AddressingMode::Register(Register8::B)),
            0x81 => Self::AddA(AddressingMode::Register(Register8::C)),
            0x82 => Self::AddA(AddressingMode::Register(Register8::D)),
            0x83 => Self::AddA(AddressingMode::Register(Register8::E)),
            0x84 => Self::AddA(AddressingMode::Register(Register8::H)),
            0x85 => Self::AddA(AddressingMode::Register(Register8::L)),
            0x86 => Self::AddA(AddressingMode::Indirect(Register16::HL)),
            0x87 => Self::AddA(AddressingMode::Register(Register8::A)),
            0xC6 => Self::AddA(AddressingMode::Immediate8),

            0x88 => Self::Adc(AddressingMode::Register(Register8::B)),
            0x89 => Self::Adc(AddressingMode::Register(Register8::C)),
            0x8A => Self::Adc(AddressingMode::Register(Register8::D)),
            0x8B => Self::Adc(AddressingMode::Register(Register8::E)),
            0x8C => Self::Adc(AddressingMode::Register(Register8::H)),
            0x8D => Self::Adc(AddressingMode::Register(Register8::L)),
            0x8E => Self::Adc(AddressingMode::Indirect(Register16::HL)),
            0x8F => Self::Adc(AddressingMode::Register(Register8::A)),
            0xCE => Self::Adc(AddressingMode::Immediate8),

            0xE8 => Self::AddSP,

            _ => unimplemented!("unimplemented opcode {:0X}", value),
        }
    }

    fn prefixed(_value: u8) -> Self {
        todo!("Parsing prefixed instruction");
    }

}
