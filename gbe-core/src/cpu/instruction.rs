use crate::cpu::{ConditionCode, registers::{Register8, Register16}};

pub enum Instruction {
    Nop,

    Adc(AddressingMode),
    AddA(AddressingMode),
    AddHL(Register16),
    AddSP,
    And(AddressingMode),
    Bit(u8, AddressingMode),
    Call(Option<ConditionCode>),
    Ccf,
    Cp(AddressingMode),
    Cpl,
    Daa,
    Dec8(Register8),
    DecHL,
    Dec16(Register16),
    Di,
    Ei,
    Halt,
    Inc8(Register8),
    IncHL,
    Inc16(Register16),
    Jp(Option<ConditionCode>),
    JpHL,
    Jr(Option<ConditionCode>),
    Ld8(AddressingMode, AddressingMode),
    Ld16(Register16, AddressingMode16),
    LdSp,
    LdHLSp,
    Or(AddressingMode),
    Pop(Register16),
    Push(Register16),
    Res(u8, AddressingMode),
    Ret(Option<ConditionCode>),
    RetI,
    Rl(AddressingMode),
    Rlc(AddressingMode),
    Rr(AddressingMode),
    Rrc(AddressingMode),
    Rst(u16),
    Sbc(AddressingMode),
    Scf,
    Set(u8, AddressingMode),
    Sla(AddressingMode),
    Sra(AddressingMode),
    Srl(AddressingMode),
    Stop,
    Sub(AddressingMode),
    Swap(AddressingMode),
    Xor(AddressingMode),   
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AddressingMode {
    Register(Register8),
    Indirect(Register16),
    IdirectImm16,
    IndirectInc(Register16),
    IndirectDec(Register16),
    IndirectImmH,
    CarryH,
    Immediate8,
}

pub enum AddressingMode16 {
    Register(Register16),
    Immediate16,
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
