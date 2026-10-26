use crate::cpu::registers::{Register16, Register8};

pub enum Operation {
    Nop,

    Adc(AddressingMode),
    AddA(AddressingMode),
    AddHL(Register16),
    AddSP,
}

pub enum AddressingMode {
    Register(Register8),
    Indirect(Register16),
    Immediate,
}

pub struct Instruction {
    pub op: Operation,
    pub cycles: u8,
}

impl Instruction {
    fn new(op: Operation, cycles: u8) -> Self {
        Self { op, cycles }
    }
}

impl From<u8> for Instruction {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::new(Operation::Nop, 1),

            0x09 => Self::new(Operation::AddHL(Register16::BC), 2),
            0x19 => Self::new(Operation::AddHL(Register16::DE), 2),
            0x29 => Self::new(Operation::AddHL(Register16::HL), 2),
            0x39 => Self::new(Operation::AddHL(Register16::SP), 2),

            0x80 => Self::new(Operation::AddA(AddressingMode::Register(Register8::B)), 1),
            0x81 => Self::new(Operation::AddA(AddressingMode::Register(Register8::C)), 1),
            0x82 => Self::new(Operation::AddA(AddressingMode::Register(Register8::D)), 1),
            0x83 => Self::new(Operation::AddA(AddressingMode::Register(Register8::E)), 1),
            0x84 => Self::new(Operation::AddA(AddressingMode::Register(Register8::H)), 1),
            0x85 => Self::new(Operation::AddA(AddressingMode::Register(Register8::L)), 1),
            0x86 => Self::new(Operation::AddA(AddressingMode::Indirect(Register16::HL)), 2),
            0x87 => Self::new(Operation::AddA(AddressingMode::Register(Register8::A)), 1),
            0xC6 => Self::new(Operation::AddA(AddressingMode::Immediate), 2),

            0x88 => Self::new(Operation::Adc(AddressingMode::Register(Register8::B)), 1),
            0x89 => Self::new(Operation::Adc(AddressingMode::Register(Register8::C)), 1),
            0x8A => Self::new(Operation::Adc(AddressingMode::Register(Register8::D)), 1),
            0x8B => Self::new(Operation::Adc(AddressingMode::Register(Register8::E)), 1),
            0x8C => Self::new(Operation::Adc(AddressingMode::Register(Register8::H)), 1),
            0x8D => Self::new(Operation::Adc(AddressingMode::Register(Register8::L)), 1),
            0x8E => Self::new(Operation::Adc(AddressingMode::Indirect(Register16::HL)), 2),
            0x8F => Self::new(Operation::Adc(AddressingMode::Register(Register8::A)), 1),
            0xCE => Self::new(Operation::Adc(AddressingMode::Immediate), 2),

            0xE8 => Self::new(Operation::AddSP, 4),

            _ => unimplemented!("unimplemented opcode {:0X}", value),
        }
    }
}
