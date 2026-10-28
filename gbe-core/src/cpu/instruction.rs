
use crate::cpu::{
    registers::{Register16, Register8},
    ConditionCode,
};

#[derive(Debug)]
pub enum Instruction {
    Nop,
    Prefixed,
    Illegal,

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AddressingMode16 {
    Register(Register16),
    Immediate16,
}

impl Instruction {
    pub fn decode(value: u8, prefixed: bool) -> Self {
        if prefixed {
            Self::prefixed(value)
        } else {
            Self::standard(value)
        }
    }

    fn standard(value: u8) -> Self {
        if (0x40..=0x7F).contains(&value) {
            if value == 0x76 {
                return Self::Halt;
            }
            return Self::Ld8(
                Self::addressing_mode8((value >> 3) & 0x07),
                Self::addressing_mode8(value & 0x07),
            );
        }

        if (0x80..=0xBF).contains(&value) {
            let mode = Self::addressing_mode8(value & 0x07);
            return match (value >> 3) & 0x07 {
                0 => Self::AddA(mode),
                1 => Self::Adc(mode),
                2 => Self::Sub(mode),
                3 => Self::Sbc(mode),
                4 => Self::And(mode),
                5 => Self::Xor(mode),
                6 => Self::Or(mode),
                7 => Self::Cp(mode),
                _ => unreachable!(),
            };
        }

        if value & 0xC7 == 0x04 {
            return match value & 0x38 {
                0x30 => Self::IncHL,
                _ => Self::Inc8(Self::register8((value >> 3) & 0x07)),
            };
        }

        if value & 0xC7 == 0x05 {
            return match value & 0x38 {
                0x30 => Self::DecHL,
                _ => Self::Dec8(Self::register8((value >> 3) & 0x07)),
            };
        }

        if value & 0xC7 == 0x06 {
            return Self::Ld8(
                Self::addressing_mode8((value >> 3) & 0x07),
                AddressingMode::Immediate8,
            );
        }

        if value & 0xCF == 0x01 {
            return Self::Ld16(
                Self::register16((value >> 4) & 0x03),
                AddressingMode16::Immediate16,
            );
        }

        if value & 0xCF == 0x03 {
            return Self::Inc16(Self::register16((value >> 4) & 0x03));
        }

        if value & 0xCF == 0x0B {
            return Self::Dec16(Self::register16((value >> 4) & 0x03));
        }

        if value & 0xCF == 0x09 {
            return Self::AddHL(Self::register16((value >> 4) & 0x03));
        }

        match value {
            0x00 => Self::Nop,
            0x02 => Self::Ld8(
                AddressingMode::Indirect(Register16::BC),
                AddressingMode::Register(Register8::A),
            ),
            0x07 => Self::Rlc(AddressingMode::Register(Register8::A)),
            0x08 => Self::LdSp,
            0x0A => Self::Ld8(
                AddressingMode::Register(Register8::A),
                AddressingMode::Indirect(Register16::BC),
            ),
            0x0F => Self::Rrc(AddressingMode::Register(Register8::A)),
            0x10 => Self::Stop,
            0x12 => Self::Ld8(
                AddressingMode::Indirect(Register16::DE),
                AddressingMode::Register(Register8::A),
            ),
            0x17 => Self::Rl(AddressingMode::Register(Register8::A)),
            0x18 => Self::Jr(None),
            0x1A => Self::Ld8(
                AddressingMode::Register(Register8::A),
                AddressingMode::Indirect(Register16::DE),
            ),
            0x1F => Self::Rr(AddressingMode::Register(Register8::A)),
            0x20 => Self::Jr(Some(ConditionCode::NZ)),
            0x22 => Self::Ld8(
                AddressingMode::IndirectInc(Register16::HL),
                AddressingMode::Register(Register8::A),
            ),
            0x27 => Self::Daa,
            0x28 => Self::Jr(Some(ConditionCode::Z)),
            0x2A => Self::Ld8(
                AddressingMode::Register(Register8::A),
                AddressingMode::IndirectInc(Register16::HL),
            ),
            0x2F => Self::Cpl,
            0x30 => Self::Jr(Some(ConditionCode::NC)),
            0x32 => Self::Ld8(
                AddressingMode::IndirectDec(Register16::HL),
                AddressingMode::Register(Register8::A),
            ),
            0x37 => Self::Scf,
            0x38 => Self::Jr(Some(ConditionCode::C)),
            0x3A => Self::Ld8(
                AddressingMode::Register(Register8::A),
                AddressingMode::IndirectDec(Register16::HL),
            ),
            0x3F => Self::Ccf,
            0xC0 => Self::Ret(Some(ConditionCode::NZ)),
            0xC1 => Self::Pop(Register16::BC),
            0xC2 => Self::Jp(Some(ConditionCode::NZ)),
            0xC3 => Self::Jp(None),
            0xC4 => Self::Call(Some(ConditionCode::NZ)),
            0xC5 => Self::Push(Register16::BC),
            0xC6 => Self::AddA(AddressingMode::Immediate8),
            0xC7 => Self::Rst(0x00),
            0xC8 => Self::Ret(Some(ConditionCode::Z)),
            0xC9 => Self::Ret(None),
            0xCA => Self::Jp(Some(ConditionCode::Z)),
            0xCB => Self::Prefixed,
            0xCC => Self::Call(Some(ConditionCode::Z)),
            0xCD => Self::Call(None),
            0xCE => Self::Adc(AddressingMode::Immediate8),
            0xCF => Self::Rst(0x08),
            0xD0 => Self::Ret(Some(ConditionCode::NC)),
            0xD1 => Self::Pop(Register16::DE),
            0xD2 => Self::Jp(Some(ConditionCode::NC)),
            0xD4 => Self::Call(Some(ConditionCode::NC)),
            0xD5 => Self::Push(Register16::DE),
            0xD6 => Self::Sub(AddressingMode::Immediate8),
            0xD7 => Self::Rst(0x10),
            0xD8 => Self::Ret(Some(ConditionCode::C)),
            0xD9 => Self::RetI,
            0xDA => Self::Jp(Some(ConditionCode::C)),
            0xDC => Self::Call(Some(ConditionCode::C)),
            0xDE => Self::Sbc(AddressingMode::Immediate8),
            0xDF => Self::Rst(0x18),
            0xE0 => Self::Ld8(
                AddressingMode::IndirectImmH,
                AddressingMode::Register(Register8::A),
            ),
            0xE1 => Self::Pop(Register16::HL),
            0xE2 => Self::Ld8(
                AddressingMode::CarryH,
                AddressingMode::Register(Register8::A),
            ),
            0xE5 => Self::Push(Register16::HL),
            0xE6 => Self::And(AddressingMode::Immediate8),
            0xE7 => Self::Rst(0x20),
            0xE8 => Self::AddSP,
            0xE9 => Self::JpHL,
            0xEA => Self::Ld8(
                AddressingMode::IdirectImm16,
                AddressingMode::Register(Register8::A),
            ),
            0xEE => Self::Xor(AddressingMode::Immediate8),
            0xEF => Self::Rst(0x28),
            0xF0 => Self::Ld8(
                AddressingMode::Register(Register8::A),
                AddressingMode::IndirectImmH,
            ),
            0xF1 => Self::Pop(Register16::AF),
            0xF2 => Self::Ld8(
                AddressingMode::Register(Register8::A),
                AddressingMode::CarryH,
            ),
            0xF3 => Self::Di,
            0xF5 => Self::Push(Register16::AF),
            0xF6 => Self::Or(AddressingMode::Immediate8),
            0xF7 => Self::Rst(0x30),
            0xF8 => Self::LdHLSp,
            0xF9 => Self::Ld16(Register16::SP, AddressingMode16::Register(Register16::HL)),
            0xFA => Self::Ld8(
                AddressingMode::Register(Register8::A),
                AddressingMode::IdirectImm16,
            ),
            0xFB => Self::Ei,
            0xFE => Self::Cp(AddressingMode::Immediate8),
            0xFF => Self::Rst(0x38),
            _ => Self::Illegal,
        }
    }

    fn prefixed(value: u8) -> Self {
        let mode = Self::addressing_mode8(value & 0x07);

        match value >> 6 {
            0 => match (value >> 3) & 0x07 {
                0 => Self::Rlc(mode),
                1 => Self::Rrc(mode),
                2 => Self::Rl(mode),
                3 => Self::Rr(mode),
                4 => Self::Sla(mode),
                5 => Self::Sra(mode),
                6 => Self::Swap(mode),
                7 => Self::Srl(mode),
                _ => unreachable!(),
            },
            1 => Self::Bit((value >> 3) & 0x07, mode),
            2 => Self::Res((value >> 3) & 0x07, mode),
            3 => Self::Set((value >> 3) & 0x07, mode),
            _ => unreachable!(),
        }
    }

    fn addressing_mode8(code: u8) -> AddressingMode {
        match code {
            6 => AddressingMode::Indirect(Register16::HL),
            _ => AddressingMode::Register(Self::register8(code)),
        }
    }

    fn register8(code: u8) -> Register8 {
        match code {
            0 => Register8::B,
            1 => Register8::C,
            2 => Register8::D,
            3 => Register8::E,
            4 => Register8::H,
            5 => Register8::L,
            7 => Register8::A,
            _ => unreachable!("opcode does not select an 8-bit register: {code}"),
        }
    }

    fn register16(code: u8) -> Register16 {
        match code {
            0 => Register16::BC,
            1 => Register16::DE,
            2 => Register16::HL,
            3 => Register16::SP,
            _ => unreachable!("opcode does not select a 16-bit register: {code}"),
        }
    }
}