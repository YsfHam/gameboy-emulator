use crate::cpu::registers::{Flag, Register8::{self, H}, Register16, Registers};

pub mod registers;
mod helpers;

pub struct Cpu {
    registers: Registers,
    pc: u16,
    cycles: u8,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            pc: 0,
            cycles: 0,
        }
    }
}




struct FlagsUpdate {
    flag_z: Option<bool>,
    flag_n: Option<bool>,
    flag_h: Option<bool>,
    flag_c: Option<bool>,
}

impl Cpu {
    fn adc(&mut self, value: u8) -> FlagsUpdate {
        let reg_a = self.registers.read_u8(Register8::A);
        let carry = self.registers.get_flag(Flag::Carry);
        let (result, half_carry, carry) = helpers::add_u8(reg_a, value, carry);
        self.registers.write_u8(Register8::A, result);

        FlagsUpdate {
            flag_z: Some(result == 0),
            flag_n: Some(false),
            flag_h: Some(half_carry),
            flag_c: Some(carry),
        }
    }

    fn add_a(&mut self, value: u8) -> FlagsUpdate {
        let reg_a = self.registers.read_u8(Register8::A);
        let (result, half_carry, carry) = helpers::add_u8(reg_a, value, false);
        self.registers.write_u8(Register8::A, result);

        FlagsUpdate {
            flag_z: Some(result == 0),
            flag_n: Some(false),
            flag_h: Some(half_carry),
            flag_c: Some(carry),
        }
    }

    fn add_hl(&mut self, value: u16) -> FlagsUpdate {
        let reg_hl = self.registers.read_u16(Register16::HL);
        let (result, half_carry, carry) = helpers::add_u16(reg_hl, value);
        self.registers.write_u16(Register16::HL, result);
        FlagsUpdate {
            flag_z: None,
            flag_n: Some(false),
            flag_h: Some(half_carry),
            flag_c: Some(carry),
        }
    }

    fn add_sp(&mut self, value: i8) -> FlagsUpdate {
        let reg_sp = self.registers.read_u16(Register16::SP);
        let result = reg_sp.wrapping_add_signed(value as i16);
        self.registers.write_u16(Register16::SP, result);

        let (_, half_carry, carry) = helpers::add_u8(reg_sp as u8, value as u8, false);
        FlagsUpdate {
            flag_z: Some(false),
            flag_n: Some(false),
            flag_h: Some(half_carry),
            flag_c: Some(carry),
        }
    }
}