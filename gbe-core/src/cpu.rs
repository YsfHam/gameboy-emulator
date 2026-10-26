use crate::{bus::Bus, cpu::registers::{Flag, Register8, Register16, Registers}, instruction::{AddressingMode, Instruction}};

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

    pub fn clock(&mut self, bus: &mut Bus) {
        if self.cycles == 0 {
            let instruction = self.fetch_instruction(bus);
            self.execute(bus, instruction);
        }
        self.cycles = self.cycles.saturating_sub(1);
    }
}


impl Cpu {
    fn fetch(&mut self, bus: &Bus) -> u8 {
        let data = bus.read_u8(self.pc);
        self.pc = self.pc.wrapping_add(1);
        data
    }

    fn fetch_instruction(&mut self, bus: &Bus) -> Instruction {
        let opcode = self.fetch(bus);
        Instruction::from(opcode)
    }

    fn get_data(&mut self, bus: &Bus, mode: AddressingMode) -> u8 {
        match mode {
            AddressingMode::Register(register8) => self.registers.read_u8(register8),
            AddressingMode::Indirect(register16) => {
                let addr = self.registers.read_u16(register16);
                bus.read_u8(addr)
            },
            AddressingMode::Immediate => self.fetch(bus),
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
    fn execute(&mut self, bus: &mut Bus, instruction: Instruction) {
        let flags_update = match instruction.op {
            crate::instruction::Operation::Nop => self.nop(),

            crate::instruction::Operation::Adc(addressing_mode) => {
                let value = self.get_data(bus, addressing_mode);
                self.adc(value)
            },
            crate::instruction::Operation::AddA(addressing_mode) => {
                let value = self.get_data(bus, addressing_mode);
                self.add_a(value)
            },
            crate::instruction::Operation::AddHL(register16) => {
                let value = self.registers.read_u16(register16);
                self.add_hl(value)
            },
            crate::instruction::Operation::AddSP => {
                let value = self.fetch(bus);
                self.add_sp(value as i8)
            },
        };
        self.cycles = instruction.cycles;
        self.update_flags(flags_update);
    }

    fn update_flags(&mut self, flags_update: FlagsUpdate) {
        let FlagsUpdate {
            flag_z,
            flag_n,
            flag_h,
            flag_c,
        } = flags_update;

        let flags = flag_z
        .map(|value| (Flag::Zero, value))
        .into_iter()
        .chain(flag_n.map(|value| (Flag::Substraction, value)).into_iter())
        .chain(flag_h.map(|value| (Flag::HalfCarry, value)).into_iter())
        .chain(flag_c.map(|value| (Flag::Carry, value)).into_iter());

        for (flag, value) in flags {
            self.registers.update_flag(flag, value);
        }
    }
}

impl Cpu {

    fn nop(&self) -> FlagsUpdate {
        FlagsUpdate {
            flag_z: None,
            flag_n: None,
            flag_h: None,
            flag_c: None,
        }
    }


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