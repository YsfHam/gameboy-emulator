use crate::{
    bus::Bus, cpu::{
        instruction::{AddressingMode, AddressingMode16, Instruction}, registers::{Flag, Register8, Register16, Registers},
    }, interrupts::Interrupt,
};

mod helpers;
pub mod instruction;
pub mod registers;

pub struct Cpu {
    registers: Registers,
    pc: u16,
    remaining_cycles: u8,
    prefixed_instruction: bool,
    ime: bool,
    ime_enable_countdown: Option<u8>,
    halt: bool,
    halt_bug: bool,
    hang: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self::with_entry_point(0)
    }

    pub fn with_entry_point(entry_point: u16) -> Self {
        Self {
            registers: Registers::new(),
            pc: entry_point,
            remaining_cycles: 0,
            prefixed_instruction: false,
            ime: false,
            ime_enable_countdown: None,
            halt: false,
            halt_bug: false,
            hang: false,
        }
    }

    pub fn clock(&mut self, bus: &mut Bus) {
        if self.hang {
            return;
        }

        if self.remaining_cycles == 0 {
            if !self.handle_interrupts(bus) {
                if self.halt {
                    return;
                }
                let instruction = self.fetch_instruction(bus);
                println!("[pc = {:06x}] Instruction {instruction:?}", self.pc - 1);
                self.execute(bus, instruction);
            }
        }
        self.remaining_cycles = self.remaining_cycles.saturating_sub(1);
    }

    fn handle_interrupts(&mut self, bus: &mut Bus) -> bool {
        self.set_ime();
        let interrupt_pending = bus.interrupts_flags.any_interrupt_pending();
        self.halt &= !interrupt_pending;
        if self.ime && interrupt_pending {
            self.ime = false;
            if self.dispatch_interrupt_handler(Interrupt::VBlank, bus) {
                true
            }
            else if self.dispatch_interrupt_handler(Interrupt::LCD, bus) {
                true
            }
            else if self.dispatch_interrupt_handler(Interrupt::Timer, bus) {
                true
            }
            else if self.dispatch_interrupt_handler(Interrupt::Serial, bus) {
                true
            }
            else if self.dispatch_interrupt_handler(Interrupt::Joypad, bus) {
                true
            }
            else {
                false
            }
        }
        else {
            false
        }
    }

    fn set_ime(&mut self) {
        match self.ime_enable_countdown {
            Some(0) => {
                self.ime = true;
                self.ime_enable_countdown = None;
            }
            Some(n) => self.ime_enable_countdown = Some(n - 1),
            None => (),
        }
    }

    fn dispatch_interrupt_handler(&mut self, interrupt: Interrupt, bus: &mut Bus) -> bool {
        if bus.interrupts_flags.is_interrupt_pending(interrupt) {
            let addr = interrupt.handler_addr();
            self.stack_push(bus, self.pc);
            self.pc = addr;
            self.remaining_cycles = 5;
            bus.interrupts_flags.disable_interrupt(interrupt);
            true
        }
        else {
            false
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ConditionCode {
    Z,
    NZ,
    C,
    NC,
}

impl Cpu {
    fn fetch_u8(&mut self, bus: &Bus) -> u8 {
        let data = bus.read_u8(self.pc);
        if !self.halt_bug {
            self.pc = self.pc.wrapping_add(1);
        } else {
            self.halt_bug = false;
        }
        data
    }

    fn fetch_u16(&mut self, bus: &Bus) -> u16 {
        let lo = self.fetch_u8(bus) as u16;
        let hi = self.fetch_u8(bus) as u16;

        (hi << 8) | lo
    }

    fn fetch_instruction(&mut self, bus: &Bus) -> Instruction {
        let opcode = self.fetch_u8(bus);
        let instruction = Instruction::decode(opcode, self.prefixed_instruction);
        self.prefixed_instruction = false;
        instruction
    }

    fn addressing_mode_read(&mut self, bus: &Bus, mode: AddressingMode) -> u8 {
        match mode {
            AddressingMode::Register(register8) => self.registers.read_u8(register8),
            AddressingMode::Indirect(register16) => {
                let addr = self.registers.read_u16(register16);
                bus.read_u8(addr)
            }
            AddressingMode::Immediate8 => self.fetch_u8(bus),
            AddressingMode::IdirectImm16 => {
                let addr = self.fetch_u16(bus);
                bus.read_u8(addr)
            }
            AddressingMode::IndirectInc(register16) => {
                let addr = self.registers.read_u16(register16);
                self.registers.write_u16(register16, addr.wrapping_add(1));
                bus.read_u8(addr)
            }
            AddressingMode::IndirectDec(register16) => {
                let addr = self.registers.read_u16(register16);
                self.registers.write_u16(register16, addr.wrapping_sub(1));
                bus.read_u8(addr)
            }
            AddressingMode::IndirectImmH => {
                let addr = 0xFF00 | self.fetch_u8(bus) as u16;
                bus.read_u8(addr)
            }
            AddressingMode::RegCIndirect => {
                let addr = 0xFF00 | self.registers.read_u8(Register8::C) as u16;
                bus.read_u8(addr)
            }
        }
    }

    fn addressing_mode_write(&mut self, bus: &mut Bus, mode: AddressingMode, value: u8) {
        match mode {
            AddressingMode::Register(register8) => self.registers.write_u8(register8, value),
            AddressingMode::Indirect(register16) => {
                let addr = self.registers.read_u16(register16);
                bus.write_u8(addr, value);
            }
            AddressingMode::IdirectImm16 => {
                let addr = self.fetch_u16(bus);
                bus.write_u8(addr, value);
            }
            AddressingMode::IndirectInc(register16) => {
                let addr = self.registers.read_u16(register16);
                self.registers.write_u16(register16, addr.wrapping_add(1));
                bus.write_u8(addr, value);
            }
            AddressingMode::IndirectDec(register16) => {
                let addr = self.registers.read_u16(register16);
                self.registers.write_u16(register16, addr.wrapping_sub(1));
                bus.write_u8(addr, value);
            }
            AddressingMode::IndirectImmH => {
                let addr = 0xFF00 | self.fetch_u8(bus) as u16;
                bus.write_u8(addr, value);
            }
            AddressingMode::RegCIndirect => {
                let addr = 0xFF00 | self.registers.read_u8(Register8::C) as u16;
                bus.write_u8(addr, value);
            }
            AddressingMode::Immediate8 => unreachable!("Cannot write to immediate addressing mode"),
        }
    }

    fn addressing_mode16_read(&mut self, bus: &Bus, mode: AddressingMode16) -> u16 {
        match mode {
            AddressingMode16::Register(register16) => self.registers.read_u16(register16),
            AddressingMode16::Immediate16 => self.fetch_u16(bus),
        }
    }

    fn alu_cycles(mode: &AddressingMode) -> u8 {
        match mode {
            AddressingMode::Register(_) => 0,
            AddressingMode::Indirect(_)
            | AddressingMode::Immediate8
            | AddressingMode::IndirectInc(_)
            | AddressingMode::IndirectDec(_) 
            | AddressingMode::RegCIndirect=> 1,
            AddressingMode::IdirectImm16 => 3,
            AddressingMode::IndirectImmH => 2,
        }
    }

    fn stack_push(&mut self, bus: &mut Bus, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = (value & 0xFF) as u8;

        let mut sp = self.registers.read_u16(Register16::SP);
        sp = sp.wrapping_sub(1);
        bus.write_u8(sp, hi);
        sp = sp.wrapping_sub(1);
        bus.write_u8(sp, lo);
        self.registers.write_u16(Register16::SP, sp);
    }

    fn stack_pop(&mut self, bus: &Bus) -> u16 {
        let mut sp = self.registers.read_u16(Register16::SP);
        let lo = bus.read_u8(sp);
        sp = sp.wrapping_add(1);
        let hi = bus.read_u8(sp);
        sp = sp.wrapping_add(1);
        self.registers.write_u16(Register16::SP, sp);
        ((hi as u16) << 8) | (lo as u16)
    }

    fn check_condition(&self, cc: ConditionCode) -> bool {
        match cc {
            ConditionCode::Z => self.registers.get_flag(Flag::Zero),
            ConditionCode::NZ => !self.registers.get_flag(Flag::Zero),
            ConditionCode::C => self.registers.get_flag(Flag::Carry),
            ConditionCode::NC => !self.registers.get_flag(Flag::Carry),
        }
    }
}

struct FlagsUpdate {
    flag_z: Option<bool>,
    flag_n: Option<bool>,
    flag_h: Option<bool>,
    flag_c: Option<bool>,
}

impl Default for FlagsUpdate {
    fn default() -> Self {
        Self {
            flag_z: None,
            flag_n: None,
            flag_h: None,
            flag_c: None,
        }
    }
}

struct ExecutionResult {
    machine_cycles: u8,
    flags_update: FlagsUpdate,
}

impl Cpu {
    fn execute(&mut self, bus: &mut Bus, instruction: Instruction) {
        let result = match instruction {
            Instruction::Nop => self.nop(),

            Instruction::Adc(addressing_mode) => {
                let cycles = Self::alu_cycles(&addressing_mode);
                let value = self.addressing_mode_read(bus, addressing_mode);
                self.adc(value, cycles)
            }
            Instruction::AddA(addressing_mode) => {
                let cycles = Self::alu_cycles(&addressing_mode);
                let value = self.addressing_mode_read(bus, addressing_mode);
                self.add_a(value, cycles)
            }
            Instruction::AddHL(register16) => {
                let value = self.registers.read_u16(register16);
                self.add_hl(value)
            }
            Instruction::AddSP => {
                let value = self.fetch_u8(bus);
                self.add_sp(value as i8)
            }
            Instruction::And(addressing_mode) => {
                let cycles = Self::alu_cycles(&addressing_mode);
                let value = self.addressing_mode_read(bus, addressing_mode);
                self.and(value, cycles)
            }
            Instruction::Bit(offset, addressing_mode) => {
                let value = self.addressing_mode_read(bus, addressing_mode);
                let alu_cycles = Self::alu_cycles(&addressing_mode);
                self.bit(offset, value, alu_cycles)
            }
            Instruction::Call(condition_code) => {
                let addr = self.fetch_u16(bus);
                match condition_code {
                    Some(cc) => self.call_cc(bus, cc, addr),
                    None => self.call(bus, addr),
                }
            }
            Instruction::Ccf => self.ccf(),
            Instruction::Cp(addressing_mode) => {
                let value = self.addressing_mode_read(bus, addressing_mode);
                let alu_cycles = Self::alu_cycles(&addressing_mode);
                self.cp(value, alu_cycles)
            }
            Instruction::Cpl => self.cpl(),
            Instruction::Daa => self.daa(),
            Instruction::Dec8(register8) => self.dec_r8(register8),
            Instruction::DecHL => self.dec_hl(bus),
            Instruction::Dec16(register16) => self.dec_r16(register16),
            Instruction::Di => self.di(),
            Instruction::Ei => self.ei(),
            Instruction::Halt => self.halt(bus.interrupts_flags.any_interrupt_pending()),
            Instruction::Inc8(register8) => self.inc_r8(register8),
            Instruction::IncHL => self.inc_hl(bus),
            Instruction::Inc16(register16) => self.inc_r16(register16),
            Instruction::Jp(condition_code) => {
                let addr = self.fetch_u16(bus);
                match condition_code {
                    Some(cc) => self.jp_cc(cc, addr),
                    None => self.jp(addr),
                }
            }
            Instruction::JpHL => self.jp_hl(),
            Instruction::Jr(condition_code) => {
                let addr = self.fetch_u8(bus);
                match condition_code {
                    Some(cc) => self.jr_cc(cc, addr),
                    None => self.jr(addr),
                }
            }
            Instruction::Ld8(dest, src) => self.ld(bus, dest, src),
            Instruction::Ld16(Register16::SP, AddressingMode16::Register(Register16::HL)) => {
                let value = self.registers.read_u16(Register16::HL);
                self.ld_r16(Register16::SP, value, 2)
            }
            Instruction::Ld16(register16, addressing_mode16) => {
                let value = self.addressing_mode16_read(bus, addressing_mode16);
                self.ld_r16(register16, value, 3)
            }
            Instruction::LdSp => {
                let addr = self.fetch_u16(bus);
                self.ld_sp(bus, addr)
            }
            Instruction::LdHLSp => {
                let value = self.fetch_u8(bus);
                self.ld_hl_sp(value as i8)
            }
            Instruction::Or(addressing_mode) => {
                let value = self.addressing_mode_read(bus, addressing_mode);
                let alu_cycles = Self::alu_cycles(&addressing_mode);
                self.or(value, alu_cycles)
            }
            Instruction::Pop(register16) => self.pop(bus, register16),
            Instruction::Push(register16) => self.push(bus, register16),
            Instruction::Res(bit, addressing_mode) => self.res(bus, bit, addressing_mode),
            Instruction::Ret(condition_code) => match condition_code {
                Some(cc) => self.ret_cc(bus, cc),
                None => self.ret(bus),
            },
            Instruction::RetI => self.reti(bus),
            Instruction::Rla => self.accumulator_rotate(bus, Self::rl),
            Instruction::Rlca => self.accumulator_rotate(bus, Self::rlc),
            Instruction::Rl(addressing_mode) => self.rl(bus, addressing_mode),
            Instruction::Rlc(addressing_mode) => self.rlc(bus, addressing_mode),
            Instruction::Rra => self.accumulator_rotate(bus, Self::rr),
            Instruction::Rrca => self.accumulator_rotate(bus, Self::rrc),
            Instruction::Rr(addressing_mode) => self.rr(bus, addressing_mode),
            Instruction::Rrc(addressing_mode) => self.rrc(bus, addressing_mode),
            Instruction::Rst(vec) => self.rst(bus, vec),
            Instruction::Sbc(addressing_mode) => {
                let value = self.addressing_mode_read(bus, addressing_mode);
                let alu_cycles = Self::alu_cycles(&addressing_mode);
                self.sbc(value, alu_cycles)
            }
            Instruction::Scf => self.scf(),
            Instruction::Set(offset, addressing_mode) => self.set(bus, offset, addressing_mode),
            Instruction::Sla(addressing_mode) => self.sla(bus, addressing_mode),
            Instruction::Sra(addressing_mode) => self.sra(bus, addressing_mode),
            Instruction::Srl(addressing_mode) => self.srl(bus, addressing_mode),
            Instruction::Stop => self.stop(bus),
            Instruction::Sub(addressing_mode) => {
                let value = self.addressing_mode_read(bus, addressing_mode);
                let alu_cycles = Self::alu_cycles(&addressing_mode);
                self.sub(value, alu_cycles)
            }
            Instruction::Swap(addressing_mode) => self.swap(bus, addressing_mode),
            Instruction::Xor(addressing_mode) => {
                let value = self.addressing_mode_read(bus, addressing_mode);
                let alu_cycles = Self::alu_cycles(&addressing_mode);
                self.xor(value, alu_cycles)
            }
            Instruction::Prefixed => {
                self.prefixed_instruction = true;
                ExecutionResult {
                    machine_cycles: 1,
                    flags_update: Default::default(),
                }
            }
            Instruction::Illegal => {
                self.hang = true;
                return;
            }
        };

        self.remaining_cycles = result.machine_cycles;
        self.update_flags(result.flags_update);
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
    fn nop(&self) -> ExecutionResult {
        ExecutionResult {
            machine_cycles: 1,
            flags_update: FlagsUpdate::default(),
        }
    }

    fn adc(&mut self, value: u8, alu_cycles: u8) -> ExecutionResult {
        let reg_a = self.registers.read_u8(Register8::A);
        let carry = self.registers.get_flag(Flag::Carry);
        let (result, half_carry, carry) = helpers::add_u8(reg_a, value, carry);
        self.registers.write_u8(Register8::A, result);

        ExecutionResult {
            machine_cycles: alu_cycles + 1,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(half_carry),
                flag_c: Some(carry),
            },
        }
    }

    fn add_a(&mut self, value: u8, alu_cycles: u8) -> ExecutionResult {
        let reg_a = self.registers.read_u8(Register8::A);
        let (result, half_carry, carry) = helpers::add_u8(reg_a, value, false);
        self.registers.write_u8(Register8::A, result);

        ExecutionResult {
            machine_cycles: alu_cycles + 1,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(half_carry),
                flag_c: Some(carry),
            },
        }
    }

    fn add_hl(&mut self, value: u16) -> ExecutionResult {
        let reg_hl = self.registers.read_u16(Register16::HL);
        let (result, half_carry, carry) = helpers::add_u16(reg_hl, value);
        self.registers.write_u16(Register16::HL, result);
        ExecutionResult {
            machine_cycles: 2,
            flags_update: FlagsUpdate {
                flag_z: None,
                flag_n: Some(false),
                flag_h: Some(half_carry),
                flag_c: Some(carry),
            },
        }
    }

    fn add_sp(&mut self, value: i8) -> ExecutionResult {
        let reg_sp = self.registers.read_u16(Register16::SP);
        let result = reg_sp.wrapping_add_signed(value as i16);
        self.registers.write_u16(Register16::SP, result);

        let (_, half_carry, carry) = helpers::add_u8(reg_sp as u8, value as u8, false);
        ExecutionResult {
            machine_cycles: 4,
            flags_update: FlagsUpdate {
                flag_z: Some(false),
                flag_n: Some(false),
                flag_h: Some(half_carry),
                flag_c: Some(carry),
            },
        }
    }

    fn and(&mut self, value: u8, alu_cycles: u8) -> ExecutionResult {
        let reg_a = self.registers.read_u8(Register8::A);
        let result = reg_a & value;
        self.registers.write_u8(Register8::A, result);
        ExecutionResult {
            machine_cycles: alu_cycles + 1,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(true),
                flag_c: Some(false),
            },
        }
    }

    fn bit(&mut self, offset: u8, value: u8, alu_cycles: u8) -> ExecutionResult {
        let result = (value.wrapping_shr(offset as u32)) & 1;

        ExecutionResult {
            machine_cycles: alu_cycles + 2,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(true),
                flag_c: None,
            },
        }
    }

    fn call(&mut self, bus: &mut Bus, addr: u16) -> ExecutionResult {
        self.stack_push(bus, self.pc);
        self.pc = addr;

        ExecutionResult {
            machine_cycles: 6,
            flags_update: FlagsUpdate::default(),
        }
    }

    fn call_cc(&mut self, bus: &mut Bus, cc: ConditionCode, addr: u16) -> ExecutionResult {
        if self.check_condition(cc) {
            self.call(bus, addr)
        } else {
            ExecutionResult {
                machine_cycles: 3,
                flags_update: FlagsUpdate::default(),
            }
        }
    }

    fn ccf(&self) -> ExecutionResult {
        let carry = self.registers.get_flag(Flag::Carry);
        ExecutionResult {
            machine_cycles: 1,
            flags_update: FlagsUpdate {
                flag_z: None,
                flag_n: Some(false),
                flag_h: Some(false),
                flag_c: Some(!carry),
            },
        }
    }

    fn cp(&self, value: u8, alu_cycles: u8) -> ExecutionResult {
        let reg_a = self.registers.read_u8(Register8::A);
        let (result, half_carry, carry) = helpers::sub_u8(reg_a, value);

        ExecutionResult {
            machine_cycles: alu_cycles + 1,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(true),
                flag_h: Some(half_carry),
                flag_c: Some(carry),
            },
        }
    }

    fn cpl(&mut self) -> ExecutionResult {
        let reg_a = self.registers.read_u8(Register8::A);
        let result = !reg_a;
        self.registers.write_u8(Register8::A, result);
        ExecutionResult {
            machine_cycles: 1,
            flags_update: FlagsUpdate {
                flag_n: Some(true),
                flag_h: Some(true),
                ..Default::default()
            },
        }
    }

    fn daa(&mut self) -> ExecutionResult {
        let reg_a = self.registers.read_u8(Register8::A);
        let half_carry = self.registers.get_flag(Flag::HalfCarry);
        let carry = self.registers.get_flag(Flag::Carry);
        let substract = self.registers.get_flag(Flag::Substraction);
        let greater_than9 = (reg_a & 0xF) > 0x9;
        let greater_than99 = reg_a > 0x99;

        let (result, carry_flag_set) = if substract {
            let mut adjustement = 0;
            if half_carry {
                adjustement += 0x6;
            }
            if carry {
                adjustement += 0x60;
            }

            let (result, _, _) = helpers::sub_u8(reg_a, adjustement);
            (result, false)
        } else {
            let mut adjustement = 0;
            let mut carry_flag_set = false;
            if half_carry || greater_than9 {
                adjustement += 0x6;
            }
            if carry || greater_than99 {
                adjustement += 0x60;
                carry_flag_set = true;
            }

            let (result, _, _) = helpers::add_u8(reg_a, adjustement, false);
            (result, carry_flag_set)
        };

        self.registers.write_u8(Register8::A, result);

        ExecutionResult {
            machine_cycles: 1,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_h: Some(false),
                flag_c: carry_flag_set.then(|| true),
                ..Default::default()
            },
        }
    }

    fn dec_r8(&mut self, reg: Register8) -> ExecutionResult {
        let value = self.registers.read_u8(reg);
        let (result, half_carry, _) = helpers::sub_u8(value, 1);
        self.registers.write_u8(reg, result);
        ExecutionResult {
            machine_cycles: 1,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(true),
                flag_h: Some(half_carry),
                ..Default::default()
            },
        }
    }

    fn dec_hl(&mut self, bus: &mut Bus) -> ExecutionResult {
        let hl = self.registers.read_u16(Register16::HL);
        let value = bus.read_u8(hl);
        let (result, half_carry, _) = helpers::sub_u8(value, 1);
        bus.write_u8(hl, result);
        ExecutionResult {
            machine_cycles: 3,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(true),
                flag_h: Some(half_carry),
                ..Default::default()
            },
        }
    }

    fn dec_r16(&mut self, reg: Register16) -> ExecutionResult {
        let value = self.registers.read_u16(reg);
        let result = value.wrapping_sub(1);
        self.registers.write_u16(reg, result);

        ExecutionResult {
            machine_cycles: 2,
            flags_update: Default::default(),
        }
    }

    fn di(&mut self) -> ExecutionResult {
        self.ime = false;
        self.ime_enable_countdown = None;

        ExecutionResult {
            machine_cycles: 1,
            flags_update: Default::default(),
        }
    }

    fn ei(&mut self) -> ExecutionResult {
        self.ime_enable_countdown = Some(1);

        ExecutionResult {
            machine_cycles: 1,
            flags_update: Default::default(),
        }
    }

    fn halt(&mut self, interrupt_pending: bool) -> ExecutionResult {
        self.halt = !interrupt_pending;
        self.halt_bug = !self.ime && interrupt_pending;
        ExecutionResult {
            machine_cycles: 1,
            flags_update: Default::default(),
        }
    }

    fn inc_r8(&mut self, reg: Register8) -> ExecutionResult {
        let value = self.registers.read_u8(reg);
        let (result, half_carry, _) = helpers::add_u8(value, 1, false);
        self.registers.write_u8(reg, result);
        ExecutionResult {
            machine_cycles: 1,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(half_carry),
                ..Default::default()
            },
        }
    }

    fn inc_hl(&mut self, bus: &mut Bus) -> ExecutionResult {
        let hl = self.registers.read_u16(Register16::HL);
        let value = bus.read_u8(hl);
        let (result, half_carry, _) = helpers::add_u8(value, 1, false);
        bus.write_u8(hl, result);
        ExecutionResult {
            machine_cycles: 3,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(half_carry),
                ..Default::default()
            },
        }
    }

    fn inc_r16(&mut self, reg: Register16) -> ExecutionResult {
        let value = self.registers.read_u16(reg);
        let result = value.wrapping_add(1);
        self.registers.write_u16(reg, result);

        ExecutionResult {
            machine_cycles: 2,
            flags_update: Default::default(),
        }
    }

    fn jp(&mut self, addr: u16) -> ExecutionResult {
        self.pc = addr;
        ExecutionResult {
            machine_cycles: 4,
            flags_update: Default::default(),
        }
    }

    fn jp_cc(&mut self, cc: ConditionCode, addr: u16) -> ExecutionResult {
        if self.check_condition(cc) {
            self.jp(addr)
        } else {
            ExecutionResult {
                machine_cycles: 3,
                flags_update: Default::default(),
            }
        }
    }

    fn jp_hl(&mut self) -> ExecutionResult {
        let hl_reg = self.registers.read_u16(Register16::HL);
        self.pc = hl_reg;
        ExecutionResult {
            machine_cycles: 1,
            flags_update: Default::default(),
        }
    }

    fn jr(&mut self, rel_addr: u8) -> ExecutionResult {
        self.pc = self.pc.wrapping_add_signed(rel_addr as i8 as i16);
        ExecutionResult {
            machine_cycles: 3,
            flags_update: Default::default(),
        }
    }

    fn jr_cc(&mut self, cc: ConditionCode, rel_addr: u8) -> ExecutionResult {
        if self.check_condition(cc) {
            self.jr(rel_addr)
        } else {
            ExecutionResult {
                machine_cycles: 2,
                flags_update: Default::default(),
            }
        }
    }

    fn ld(&mut self, bus: &mut Bus, dest: AddressingMode, src: AddressingMode) -> ExecutionResult {
        let value = self.addressing_mode_read(bus, src);
        self.addressing_mode_write(bus, dest, value);
        ExecutionResult {
            machine_cycles: Self::alu_cycles(&src) + Self::alu_cycles(&dest) + 1,
            flags_update: Default::default(),
        }
    }

    fn ld_r16(&mut self, reg: Register16, value: u16, machine_cycles: u8) -> ExecutionResult {
        self.registers.write_u16(reg, value);
        ExecutionResult {
            machine_cycles,
            flags_update: Default::default(),
        }
    }

    fn ld_sp(&self, bus: &mut Bus, addr: u16) -> ExecutionResult {
        let sp = self.registers.read_u16(Register16::SP);
        bus.write_u8(addr, (sp & 0xFF) as u8);
        bus.write_u8(addr.wrapping_add(1), (sp >> 8) as u8);
        ExecutionResult {
            machine_cycles: 5,
            flags_update: Default::default(),
        }
    }

    fn ld_hl_sp(&mut self, value: i8) -> ExecutionResult {
        let sp = self.registers.read_u16(Register16::SP);
        let result = sp.wrapping_add_signed(value as i16);
        self.registers.write_u16(Register16::HL, result);

        let (_, half_carry, carry) = helpers::add_u8((sp & 0xFf) as u8, value as u8, false);

        ExecutionResult {
            machine_cycles: 3,
            flags_update: FlagsUpdate {
                flag_z: Some(false),
                flag_n: Some(false),
                flag_h: Some(half_carry),
                flag_c: Some(carry),
            },
        }
    }

    fn or(&mut self, value: u8, alu_cycles: u8) -> ExecutionResult {
        let reg_a = self.registers.read_u8(Register8::A);
        let result = reg_a | value;
        self.registers.write_u8(Register8::A, result);
        ExecutionResult {
            machine_cycles: alu_cycles + 1,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(false),
                flag_c: Some(false),
            },
        }
    }

    fn pop(&mut self, bus: &Bus, reg: Register16) -> ExecutionResult {
        let value = self.stack_pop(bus);
        self.registers.write_u16(reg, value);
        ExecutionResult {
            machine_cycles: 3,
            flags_update: Default::default(),
        }
    }

    fn push(&mut self, bus: &mut Bus, reg: Register16) -> ExecutionResult {
        let value = self.registers.read_u16(reg);
        self.stack_push(bus, value);
        ExecutionResult {
            machine_cycles: 4,
            flags_update: Default::default(),
        }
    }

    fn res(
        &mut self,
        bus: &mut Bus,
        offset: u8,
        addressing_mode: AddressingMode,
    ) -> ExecutionResult {
        let value = self.addressing_mode_read(bus, addressing_mode);
        let result = value & !(1 << offset);
        self.addressing_mode_write(bus, addressing_mode, result);
        ExecutionResult {
            machine_cycles: 2 + Self::alu_cycles(&addressing_mode) * 2,
            flags_update: Default::default(),
        }
    }

    fn ret(&mut self, bus: &Bus) -> ExecutionResult {
        self.pc = self.stack_pop(bus);

        ExecutionResult {
            machine_cycles: 4,
            flags_update: Default::default(),
        }
    }

    fn ret_cc(&mut self, bus: &Bus, cc: ConditionCode) -> ExecutionResult {
        if self.check_condition(cc) {
            self.pc = self.stack_pop(bus);
            ExecutionResult {
                machine_cycles: 5,
                flags_update: Default::default(),
            }
        } else {
            ExecutionResult {
                machine_cycles: 2,
                flags_update: Default::default(),
            }
        }
    }

    fn accumulator_rotate(
        &mut self,
        bus: &mut Bus,
        rotate: fn(&mut Self, &mut Bus, AddressingMode) -> ExecutionResult,
    ) -> ExecutionResult {
        let mut result = rotate(self, bus, AddressingMode::Register(Register8::A));
        result.machine_cycles = 1;
        result.flags_update.flag_z = Some(false);
        result
    }

    fn reti(&mut self, bus: &Bus) -> ExecutionResult {
        self.ime = true;
        self.ret(bus)
    }

    fn rl(&mut self, bus: &mut Bus, addressing_mode: AddressingMode) -> ExecutionResult {
        let value = self.addressing_mode_read(bus, addressing_mode);
        let (result, carry) = helpers::rotate_left(value, self.registers.get_flag(Flag::Carry));
        self.addressing_mode_write(bus, addressing_mode, result);
        ExecutionResult {
            machine_cycles: 2 + Self::alu_cycles(&addressing_mode) * 2,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(false),
                flag_c: Some(carry),
            },
        }
    }

    fn rlc(&mut self, bus: &mut Bus, addressing_mode: AddressingMode) -> ExecutionResult {
        let value = self.addressing_mode_read(bus, addressing_mode);
        let (result, carry) = helpers::circular_left_rotation(value);
        self.addressing_mode_write(bus, addressing_mode, result);

        ExecutionResult {
            machine_cycles: 2 + Self::alu_cycles(&addressing_mode) * 2,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(false),
                flag_c: Some(carry),
            },
        }
    }

    fn rr(&mut self, bus: &mut Bus, addressing_mode: AddressingMode) -> ExecutionResult {
        let value = self.addressing_mode_read(bus, addressing_mode);
        let (result, carry) = helpers::rotate_right(value, self.registers.get_flag(Flag::Carry));
        self.addressing_mode_write(bus, addressing_mode, result);

        ExecutionResult {
            machine_cycles: 2 + Self::alu_cycles(&addressing_mode) * 2,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(false),
                flag_c: Some(carry),
            },
        }
    }

    fn rrc(&mut self, bus: &mut Bus, addressing_mode: AddressingMode) -> ExecutionResult {
        let value = self.addressing_mode_read(bus, addressing_mode);
        let (result, carry) = helpers::circular_right_rotation(value);
        self.addressing_mode_write(bus, addressing_mode, result);

        ExecutionResult {
            machine_cycles: 2 + Self::alu_cycles(&addressing_mode) * 2,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(false),
                flag_c: Some(carry),
            },
        }
    }

    fn rst(&mut self, bus: &mut Bus, vec: u16) -> ExecutionResult {
        self.stack_push(bus, self.pc);
        self.pc = vec;

        ExecutionResult {
            machine_cycles: 4,
            flags_update: Default::default(),
        }
    }

    fn sbc(&mut self, value: u8, alu_cycles: u8) -> ExecutionResult {
        let reg_a = self.registers.read_u8(Register8::A);
        let (result, half_carry, carry) =
            helpers::carrying_sub_u8(reg_a, value, self.registers.get_flag(Flag::Carry));
        self.registers.write_u8(Register8::A, result);
        ExecutionResult {
            machine_cycles: alu_cycles + 1,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(true),
                flag_h: Some(half_carry),
                flag_c: Some(carry),
            },
        }
    }

    fn scf(&mut self) -> ExecutionResult {
        ExecutionResult {
            machine_cycles: 1,
            flags_update: FlagsUpdate {
                flag_n: Some(false),
                flag_h: Some(false),
                flag_c: Some(true),
                ..Default::default()
            },
        }
    }

    fn set(&mut self, bus: &mut Bus, bit: u8, addressing_mode: AddressingMode) -> ExecutionResult {
        let value = self.addressing_mode_read(bus, addressing_mode);
        let result = value | (1 << bit);
        self.addressing_mode_write(bus, addressing_mode, result);

        ExecutionResult {
            machine_cycles: 2 + Self::alu_cycles(&addressing_mode) * 2,
            flags_update: Default::default(),
        }
    }

    fn sla(&mut self, bus: &mut Bus, addressing_mode: AddressingMode) -> ExecutionResult {
        let value = self.addressing_mode_read(bus, addressing_mode);
        let (result, carry) = helpers::shift_left(value);
        self.addressing_mode_write(bus, addressing_mode, result);

        ExecutionResult {
            machine_cycles: 2 + Self::alu_cycles(&addressing_mode) * 2,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(false),
                flag_c: Some(carry),
            },
        }
    }

    fn sra(&mut self, bus: &mut Bus, addressing_mode: AddressingMode) -> ExecutionResult {
        let value = self.addressing_mode_read(bus, addressing_mode);
        let (result, carry) = helpers::arithmetic_shift_right(value);
        self.addressing_mode_write(bus, addressing_mode, result);

        ExecutionResult {
            machine_cycles: 2 + Self::alu_cycles(&addressing_mode) * 2,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(false),
                flag_c: Some(carry),
            },
        }
    }

    fn srl(&mut self, bus: &mut Bus, addressing_mode: AddressingMode) -> ExecutionResult {
        let value = self.addressing_mode_read(bus, addressing_mode);
        let (result, carry) = helpers::shift_right(value);
        self.addressing_mode_write(bus, addressing_mode, result);

        ExecutionResult {
            machine_cycles: 2 + Self::alu_cycles(&addressing_mode) * 2,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(false),
                flag_c: Some(carry),
            },
        }
    }

    fn stop(&mut self, _bus: &Bus) -> ExecutionResult {
        todo!("Implement stop")
    }

    fn sub(&mut self, value: u8, alu_cycles: u8) -> ExecutionResult {
        let reg_a = self.registers.read_u8(Register8::A);
        let (result, half_carry, carry) = helpers::sub_u8(reg_a, value);
        self.registers.write_u8(Register8::A, result);

        ExecutionResult {
            machine_cycles: alu_cycles + 1,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(true),
                flag_h: Some(half_carry),
                flag_c: Some(carry),
            },
        }
    }

    fn swap(&mut self, bus: &mut Bus, addressing_mode: AddressingMode) -> ExecutionResult {
        let value = self.addressing_mode_read(bus, addressing_mode);
        let result = helpers::swap_nibbles(value);
        self.addressing_mode_write(bus, addressing_mode, result);

        ExecutionResult {
            machine_cycles: 2 + Self::alu_cycles(&addressing_mode) * 2,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(false),
                flag_c: Some(false),
            },
        }
    }

    fn xor(&mut self, value: u8, alu_cycles: u8) -> ExecutionResult {
        let reg_a = self.registers.read_u8(Register8::A);
        let result = reg_a ^ value;
        self.registers.write_u8(Register8::A, result);
        ExecutionResult {
            machine_cycles: alu_cycles + 1,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(false),
                flag_c: Some(false),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cartridge::Cartridge;

use super::*;

    const TEST_PC: u16 = 0xFF80;
    const TEST_DATA: u16 = 0xC000;

    fn setup() -> (Cpu, Bus) {
        let mut cpu = Cpu::new();
        let bus = Bus::new(Cartridge::dummy());
        cpu.pc = TEST_PC;
        cpu.registers.write_u16(Register16::SP, 0xD000);
        (cpu, bus)
    }

    fn run(cpu: &mut Cpu, bus: &mut Bus, bytes: &[u8]) {
        for (offset, byte) in bytes.iter().enumerate() {
            bus.write_u8(TEST_PC.wrapping_add(offset as u16), *byte);
        }
        cpu.clock(bus);
    }

    fn run_cb(cpu: &mut Cpu, bus: &mut Bus, opcode: u8) {
        bus.write_u8(TEST_PC, 0xCB);
        bus.write_u8(TEST_PC + 1, opcode);
        cpu.clock(bus);
        cpu.clock(bus);
    }

    fn flags(cpu: &Cpu) -> (bool, bool, bool, bool) {
        (
            cpu.registers.get_flag(Flag::Zero),
            cpu.registers.get_flag(Flag::Substraction),
            cpu.registers.get_flag(Flag::HalfCarry),
            cpu.registers.get_flag(Flag::Carry),
        )
    }

    fn assert_cycles(cpu: &Cpu, machine_cycles: u8) {
        assert_eq!(cpu.remaining_cycles + 1, machine_cycles);
    }

    fn expected_base_cycles(opcode: u8) -> Option<u8> {
        if (0x40..=0x7F).contains(&opcode) {
            return Some(if opcode == 0x76 {
                1
            } else if (opcode & 0x07 == 6) || (opcode >> 3 & 0x07 == 6) {
                2
            } else {
                1
            });
        }

        if (0x80..=0xBF).contains(&opcode) {
            return Some(if opcode & 0x07 == 6 { 2 } else { 1 });
        }

        if opcode & 0xC7 == 0x04 || opcode & 0xC7 == 0x05 {
            return Some(if (opcode >> 3) & 0x07 == 6 { 3 } else { 1 });
        }

        if opcode & 0xC7 == 0x06 {
            return Some(if (opcode >> 3) & 0x07 == 6 { 3 } else { 2 });
        }

        if opcode & 0xCF == 0x01 {
            return Some(3);
        }

        if opcode & 0xCF == 0x03 || opcode & 0xCF == 0x0B {
            return Some(2);
        }

        if opcode & 0xCF == 0x09 {
            return Some(2);
        }

        Some(match opcode {
            0x00 | 0x07 | 0x0F | 0x17 | 0x1F | 0x27 | 0x2F | 0x37 | 0x3F | 0xF3 | 0xFB => 1,
            0x02 | 0x0A | 0x12 | 0x1A | 0x22 | 0x2A | 0x32 | 0x3A | 0xE2 | 0xF2 => 2,
            0x08 => 5,
            0x18 => 3,
            0x20 | 0x38 => 2,
            0x28 => 3,
            0x30 => 3,
            0xC0 => 2,
            0xD0 => 5,
            0xC1 | 0xD1 | 0xE1 | 0xF1 => 3,
            0xC2 | 0xDA => 3,
            0xD2 | 0xCA => 4,
            0xC3 => 4,
            0xC4 | 0xDC => 3,
            0xD4 | 0xCC => 6,
            0xC5 | 0xD5 | 0xE5 | 0xF5 => 4,
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => 2,
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => 4,
            0xC8 => 5,
            0xD8 => 2,
            0xC9 | 0xD9 => 4,
            0xCD => 6,
            0xCB => 1,
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                return None
            }
            0xE0 | 0xF0 => 3,
            0xEA | 0xFA => 4,
            0xE8 => 4,
            0xE9 => 1,
            0xF8 => 3,
            0xF9 => 2,
            _ => return None,
        })
    }

    fn expected_cb_cycles(opcode: u8) -> u8 {
        match opcode >> 6 {
            1 if opcode & 0x07 == 6 => 3,
            0 | 2 | 3 if opcode & 0x07 == 6 => 4,
            _ => 2,
        }
    }

    #[test]
    fn every_documented_opcode_executes_without_panicking_except_stop() {
        const ILLEGAL: [u8; 11] = [
            0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
        ];

        for opcode in 0..=u8::MAX {
            if opcode == 0x10 || ILLEGAL.contains(&opcode) {
                continue;
            }

            let (mut cpu, mut bus) = setup();
            run(&mut cpu, &mut bus, &[opcode, 0, 0]);
            assert!(
                cpu.hang
                    || cpu.halt
                    || cpu.pc != TEST_PC
                    || cpu.remaining_cycles > 0
                    || cpu.prefixed_instruction,
                "base opcode 0x{opcode:02X} did not execute"
            );
        }

        for opcode in 0..=u8::MAX {
            let (mut cpu, mut bus) = setup();
            run_cb(&mut cpu, &mut bus, opcode);
            assert!(
                cpu.pc != TEST_PC || cpu.remaining_cycles > 0,
                "CB opcode 0x{opcode:02X} did not execute"
            );
        }
    }

    #[test]
    fn documented_opcodes_match_reference_machine_cycle_counts() {
        const ILLEGAL: [u8; 11] = [
            0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD,
        ];
        let mut mismatches = Vec::new();

        for opcode in 0..=u8::MAX {
            if opcode == 0x10 || ILLEGAL.contains(&opcode) {
                continue;
            }

            let Some(expected) = expected_base_cycles(opcode) else {
                mismatches.push(format!(
                    "no cycle expectation for base opcode 0x{opcode:02X}"
                ));
                continue;
            };
            let (mut cpu, mut bus) = setup();
            cpu.registers.write_u8(Register8::A, 0x80);
            cpu.registers.write_u8(Register8::B, 1);
            cpu.registers.write_u8(Register8::F, 0x80);
            let execution = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run(&mut cpu, &mut bus, &[opcode, 0, 0]);
            }));
            if execution.is_err() {
                mismatches.push(format!("base opcode 0x{opcode:02X} panicked"));
                continue;
            }
            let actual = cpu.remaining_cycles + 1;
            if actual != expected {
                mismatches.push(format!(
                    "base opcode 0x{opcode:02X}: expected {expected}, got {actual}"
                ));
            }
        }

        for opcode in 0..=u8::MAX {
            let (mut cpu, mut bus) = setup();
            run_cb(&mut cpu, &mut bus, opcode);
            let expected = expected_cb_cycles(opcode);
            // The prefix consumes the first clock; the second clock has advanced the CB
            // instruction by one cycle, so include only its remaining cycles plus that one.
            let actual = cpu.remaining_cycles + 1;
            if actual != expected {
                mismatches.push(format!(
                    "CB opcode 0x{opcode:02X}: expected {expected}, got {actual}"
                ));
            }
        }

        assert!(
            mismatches.is_empty(),
            "instruction cycle mismatches:\n{}",
            mismatches.join("\n")
        );
    }

    #[test]
    fn conditional_control_flow_matches_taken_and_untaken_cycle_counts() {
        const CONDITIONALS: [(u8, bool, bool); 16] = [
            (0x20, true, false),
            (0x28, true, true),
            (0x30, false, false),
            (0x38, false, true),
            (0xC0, true, false),
            (0xC2, true, false),
            (0xC4, true, false),
            (0xC8, true, true),
            (0xCA, true, true),
            (0xCC, true, true),
            (0xD0, false, false),
            (0xD2, false, false),
            (0xD4, false, false),
            (0xD8, false, true),
            (0xDA, false, true),
            (0xDC, false, true),
        ];
        let mut mismatches = Vec::new();

        for (opcode, tests_zero, set_flag_when_taken) in CONDITIONALS {
            for taken in [false, true] {
                let mut flag_register = 0;
                let flag_set = taken == set_flag_when_taken;
                if flag_set {
                    flag_register |= if tests_zero { 0x80 } else { 0x10 };
                }

                let (mut cpu, mut bus) = setup();
                cpu.registers.write_u8(Register8::F, flag_register);
                let expected = match opcode {
                    0x20 | 0x28 | 0x30 | 0x38 => {
                        if taken {
                            3
                        } else {
                            2
                        }
                    }
                    0xC0 | 0xC8 | 0xD0 | 0xD8 => {
                        if taken {
                            5
                        } else {
                            2
                        }
                    }
                    0xC2 | 0xCA | 0xD2 | 0xDA => {
                        if taken {
                            4
                        } else {
                            3
                        }
                    }
                    _ => {
                        if taken {
                            6
                        } else {
                            3
                        }
                    }
                };
                let bytes: &[u8] = if matches!(opcode, 0x20 | 0x28 | 0x30 | 0x38) {
                    &[opcode, 0]
                } else {
                    &[opcode, 0, 0]
                };
                run(&mut cpu, &mut bus, &bytes);
                let actual = cpu.remaining_cycles + 1;
                if actual != expected {
                    mismatches.push(format!(
                        "opcode 0x{opcode:02X}, taken={taken}: expected {expected}, got {actual}"
                    ));
                }
            }
        }

        assert!(
            mismatches.is_empty(),
            "conditional instruction cycle mismatches:\n{}",
            mismatches.join("\n")
        );
    }

    #[test]
    fn add_sub_and_compare_update_results_and_flags() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0x0F);
        cpu.registers.write_u8(Register8::B, 0x01);
        run(&mut cpu, &mut bus, &[0x80]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0x10);
        assert_eq!(flags(&cpu), (false, false, true, false));
        assert_cycles(&cpu, 1);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0);
        cpu.registers.write_u8(Register8::B, 1);
        run(&mut cpu, &mut bus, &[0x90]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0xFF);
        assert_eq!(flags(&cpu), (false, true, true, true));

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0x20);
        cpu.registers.write_u8(Register8::B, 0x20);
        run(&mut cpu, &mut bus, &[0xB8]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0x20);
        assert_eq!(flags(&cpu), (true, true, false, false));
    }

    #[test]
    fn adc_includes_carry_in_half_carry_calculation() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0x0F);
        cpu.registers.write_u8(Register8::B, 0);
        cpu.registers.set_flag(Flag::Carry);
        run(&mut cpu, &mut bus, &[0x88]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0x10);
        assert_eq!(flags(&cpu), (false, false, true, false));
    }

    #[test]
    fn sbc_writes_subtraction_result_to_accumulator() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0x10);
        cpu.registers.write_u8(Register8::B, 1);
        cpu.registers.set_flag(Flag::Carry);
        run(&mut cpu, &mut bus, &[0x98]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0x0E);
        assert_eq!(flags(&cpu), (false, true, true, false));
        assert_cycles(&cpu, 1);
    }

    #[test]
    fn bitwise_alu_instructions_have_documented_flags() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0xF0);
        cpu.registers.write_u8(Register8::B, 0x0F);
        run(&mut cpu, &mut bus, &[0xA0]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0);
        assert_eq!(flags(&cpu), (true, false, true, false));

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0x55);
        cpu.registers.write_u8(Register8::B, 0x55);
        run(&mut cpu, &mut bus, &[0xA8]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0);
        assert_eq!(flags(&cpu), (true, false, false, false));
    }

    #[test]
    fn or_clears_half_carry() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0xF0);
        cpu.registers.write_u8(Register8::B, 0x0F);
        run(&mut cpu, &mut bus, &[0xB0]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0xFF);
        assert_eq!(flags(&cpu), (false, false, false, false));
    }

    #[test]
    fn increment_and_decrement_preserve_carry_and_handle_memory() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::B, 0x0F);
        cpu.registers.set_flag(Flag::Carry);
        run(&mut cpu, &mut bus, &[0x04]);
        assert_eq!(cpu.registers.read_u8(Register8::B), 0x10);
        assert_eq!(flags(&cpu), (false, false, true, true));

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::C, 1);
        cpu.registers.set_flag(Flag::Carry);
        run(&mut cpu, &mut bus, &[0x0D]);
        assert_eq!(cpu.registers.read_u8(Register8::C), 0);
        assert_eq!(flags(&cpu), (true, true, false, true));
    }

    #[test]
    fn increment_and_decrement_hl_memory_values_and_cycles() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::HL, TEST_DATA);
        bus.write_u8(TEST_DATA, 0xFF);
        run(&mut cpu, &mut bus, &[0x34]);
        assert_eq!(bus.read_u8(TEST_DATA), 0);
        assert_eq!(flags(&cpu), (true, false, true, false));
        assert_cycles(&cpu, 3);
    }

    #[test]
    fn decrement_hl_memory_value_and_cycles() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::HL, TEST_DATA);
        bus.write_u8(TEST_DATA, 0);
        run(&mut cpu, &mut bus, &[0x35]);
        assert_eq!(bus.read_u8(TEST_DATA), 0xFF);
        assert_eq!(flags(&cpu), (false, true, true, false));
        assert_cycles(&cpu, 3);
    }

    #[test]
    fn sixteen_bit_arithmetic_and_signed_sp_additions() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::HL, 0x0FFF);
        cpu.registers.write_u16(Register16::BC, 1);
        run(&mut cpu, &mut bus, &[0x09]);
        assert_eq!(cpu.registers.read_u16(Register16::HL), 0x1000);
        assert_eq!(flags(&cpu), (false, false, true, false));
        assert_cycles(&cpu, 2);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::SP, 0xFFF0);
        run(&mut cpu, &mut bus, &[0xE8, 0x20]);
        assert_eq!(cpu.registers.read_u16(Register16::SP), 0x0010);
        assert_eq!(flags(&cpu), (false, false, false, true));
        assert_cycles(&cpu, 4);
    }

    #[test]
    fn load_sp_from_hl_takes_two_machine_cycles() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::HL, 0x1234);
        run(&mut cpu, &mut bus, &[0xF9]);
        assert_eq!(cpu.registers.read_u16(Register16::SP), 0x1234);
        assert_cycles(&cpu, 2);
    }

    #[test]
    fn load_hl_from_signed_sp_offset_sets_flags_from_unsigned_offset() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::SP, 0x0001);
        run(&mut cpu, &mut bus, &[0xF8, 0xFF]);
        assert_eq!(cpu.registers.read_u16(Register16::HL), 0);
        assert_eq!(flags(&cpu), (false, false, true, true));
        assert_cycles(&cpu, 3);
    }

    #[test]
    fn sixteen_bit_load_increment_and_decrement_preserve_flags() {
        let (mut cpu, mut bus) = setup();
        run(&mut cpu, &mut bus, &[0x01, 0x34, 0x12]);
        assert_eq!(cpu.registers.read_u16(Register16::BC), 0x1234);
        assert_cycles(&cpu, 3);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::BC, 0xFFFF);
        cpu.registers.write_u8(Register8::F, 0xB0);
        run(&mut cpu, &mut bus, &[0x03]);
        assert_eq!(cpu.registers.read_u16(Register16::BC), 0);
        assert_eq!(flags(&cpu), (true, false, true, true));
        assert_cycles(&cpu, 2);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::SP, 0);
        cpu.registers.write_u8(Register8::F, 0xB0);
        run(&mut cpu, &mut bus, &[0x3B]);
        assert_eq!(cpu.registers.read_u16(Register16::SP), 0xFFFF);
        assert_eq!(flags(&cpu), (true, false, true, true));
        assert_cycles(&cpu, 2);
    }

    #[test]
    fn loads_cover_register_memory_immediate_and_high_memory_modes() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::B, 0x5A);
        run(&mut cpu, &mut bus, &[0x48]);
        assert_eq!(cpu.registers.read_u8(Register8::C), 0x5A);
        assert_cycles(&cpu, 1);

        let (mut cpu, mut bus) = setup();
        run(&mut cpu, &mut bus, &[0x06, 0xA5]);
        assert_eq!(cpu.registers.read_u8(Register8::B), 0xA5);
        assert_eq!(cpu.pc, TEST_PC + 2);
        assert_cycles(&cpu, 2);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::HL, TEST_DATA);
        cpu.registers.write_u8(Register8::A, 0x42);
        run(&mut cpu, &mut bus, &[0x22]);
        assert_eq!(bus.read_u8(TEST_DATA), 0x42);
        assert_eq!(cpu.registers.read_u16(Register16::HL), TEST_DATA + 1);
        assert_cycles(&cpu, 2);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::HL, TEST_DATA);
        bus.write_u8(TEST_DATA, 0x24);
        run(&mut cpu, &mut bus, &[0x2A]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0x24);
        assert_eq!(cpu.registers.read_u16(Register16::HL), TEST_DATA + 1);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0xB6);
        run(&mut cpu, &mut bus, &[0xE0, 0x80]);
        assert_eq!(bus.read_u8(0xFF00 + 0x80), 0xB6);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0x93);
        cpu.registers.write_u8(Register8::C, 0x81);
        run(&mut cpu, &mut bus, &[0xE2]);
        assert_eq!(bus.read_u8(0xFF81), 0x93);
    }

    #[test]
    fn accumulator_loads_use_absolute_addresses() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0x6C);
        run(&mut cpu, &mut bus, &[0xEA, 0x00, 0xC0]);
        assert_eq!(bus.read_u8(TEST_DATA), 0x6C);
        assert_cycles(&cpu, 4);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::HL, TEST_DATA);
        bus.write_u8(TEST_DATA, 0xD2);
        run(&mut cpu, &mut bus, &[0xFA, 0x00, 0xC0]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0xD2);
        assert_eq!(cpu.pc, TEST_PC + 3);
        assert_cycles(&cpu, 4);
    }

    #[test]
    fn load_sp_store_and_pop_push_use_correct_byte_order() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::SP, 0xBEEF);
        run(&mut cpu, &mut bus, &[0x08, 0x00, 0xC0]);
        assert_eq!(bus.read_u8(TEST_DATA), 0xEF);
        assert_eq!(bus.read_u8(TEST_DATA + 1), 0xBE);
        assert_cycles(&cpu, 5);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::SP, TEST_DATA);
        bus.write_u8(TEST_DATA, 0x34);
        bus.write_u8(TEST_DATA + 1, 0x12);
        run(&mut cpu, &mut bus, &[0xC1]);
        assert_eq!(cpu.registers.read_u16(Register16::BC), 0x1234);
        assert_eq!(cpu.registers.read_u16(Register16::SP), TEST_DATA + 2);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::SP, 0xD000);
        cpu.registers.write_u16(Register16::BC, 0x1234);
        run(&mut cpu, &mut bus, &[0xC5]);
        assert_eq!(cpu.registers.read_u16(Register16::SP), 0xCFFE);
        assert_eq!(bus.read_u8(0xCFFE), 0x34);
        assert_eq!(bus.read_u8(0xCFFF), 0x12);
        assert_cycles(&cpu, 4);
    }

    #[test]
    fn pop_af_discards_unused_low_flag_bits() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::SP, TEST_DATA);
        bus.write_u8(TEST_DATA, 0xFF);
        bus.write_u8(TEST_DATA + 1, 0x12);
        run(&mut cpu, &mut bus, &[0xF1]);
        assert_eq!(cpu.registers.read_u16(Register16::AF), 0x12F0);
    }

    #[test]
    fn conditional_and_unconditional_jumps_calls_and_returns() {
        let (mut cpu, mut bus) = setup();
        run(&mut cpu, &mut bus, &[0xC3, 0x34, 0x12]);
        assert_eq!(cpu.pc, 0x1234);
        assert_cycles(&cpu, 4);

        let (mut cpu, mut bus) = setup();
        cpu.registers.set_flag(Flag::Zero);
        run(&mut cpu, &mut bus, &[0x20, 0x02]);
        assert_eq!(cpu.pc, TEST_PC + 2);
        assert_cycles(&cpu, 2);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::SP, 0xD000);
        run(&mut cpu, &mut bus, &[0xCD, 0x00, 0xC0]);
        assert_eq!(cpu.pc, 0xC000);
        assert_eq!(cpu.registers.read_u16(Register16::SP), 0xCFFE);
        assert_eq!(bus.read_u8(0xCFFE), 0x83);
        assert_eq!(bus.read_u8(0xCFFF), 0xFF);
        assert_cycles(&cpu, 6);

        let (mut cpu, mut bus) = setup();
        run(&mut cpu, &mut bus, &[0x20, 0xFE]);
        assert_eq!(cpu.pc, TEST_PC);
        assert_cycles(&cpu, 3);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::SP, TEST_DATA);
        bus.write_u8(TEST_DATA, 0x34);
        bus.write_u8(TEST_DATA + 1, 0x12);
        run(&mut cpu, &mut bus, &[0xC9]);
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.registers.read_u16(Register16::SP), TEST_DATA + 2);
    }

    #[test]
    fn restart_and_conditional_return_behaviors() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::SP, 0xD000);
        run(&mut cpu, &mut bus, &[0xFF]);
        assert_eq!(cpu.pc, 0x0038);
        assert_eq!(bus.read_u8(0xCFFE), 0x81);
        assert_eq!(bus.read_u8(0xCFFF), 0xFF);
        assert_cycles(&cpu, 4);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::SP, TEST_DATA);
        cpu.registers.set_flag(Flag::Zero);
        run(&mut cpu, &mut bus, &[0xC0]);
        assert_eq!(cpu.pc, TEST_PC + 1);
        assert_cycles(&cpu, 2);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::SP, TEST_DATA);
        cpu.registers.set_flag(Flag::Zero);
        bus.write_u8(TEST_DATA, 0x78);
        bus.write_u8(TEST_DATA + 1, 0x56);
        run(&mut cpu, &mut bus, &[0xC8]);
        assert_eq!(cpu.pc, 0x5678);
        assert_cycles(&cpu, 5);
    }

    #[test]
    fn accumulator_rotates_and_cb_rotates_have_distinct_zero_flag_behavior() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0);
        run(&mut cpu, &mut bus, &[0x07]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0);
        assert_eq!(flags(&cpu), (false, false, false, false));
        assert_cycles(&cpu, 1);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0);
        run_cb(&mut cpu, &mut bus, 0x07);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0);
        assert_eq!(flags(&cpu), (true, false, false, false));
        assert_cycles(&cpu, 2);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::B, 0x80);
        run_cb(&mut cpu, &mut bus, 0x00);
        assert_eq!(cpu.registers.read_u8(Register8::B), 1);
        assert_eq!(flags(&cpu), (false, false, false, true));
        assert_cycles(&cpu, 2);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::HL, TEST_DATA);
        bus.write_u8(TEST_DATA, 0x80);
        run_cb(&mut cpu, &mut bus, 0x06);
        assert_eq!(bus.read_u8(TEST_DATA), 1);
        assert_eq!(flags(&cpu), (false, false, false, true));
        assert_cycles(&cpu, 4);
    }

    #[test]
    fn accumulator_rotate_through_carry_and_circular_rotate_behaviors() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0x01);
        run(&mut cpu, &mut bus, &[0x0F]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0x80);
        assert_eq!(flags(&cpu), (false, false, false, true));

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0x80);
        cpu.registers.set_flag(Flag::Carry);
        run(&mut cpu, &mut bus, &[0x17]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 1);
        assert_eq!(flags(&cpu), (false, false, false, true));

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0x01);
        cpu.registers.set_flag(Flag::Carry);
        run(&mut cpu, &mut bus, &[0x1F]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0x80);
        assert_eq!(flags(&cpu), (false, false, false, true));
    }

    #[test]
    fn cb_bit_res_set_and_shift_families_update_operands_and_flags() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::B, 0x01);
        cpu.registers.set_flag(Flag::Carry);
        run_cb(&mut cpu, &mut bus, 0x40);
        assert_eq!(flags(&cpu), (false, false, true, true));
        assert_cycles(&cpu, 2);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::B, 0xFF);
        run_cb(&mut cpu, &mut bus, 0x80);
        assert_eq!(cpu.registers.read_u8(Register8::B), 0xFE);
        assert_eq!(flags(&cpu), (false, false, false, false));

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::B, 0);
        run_cb(&mut cpu, &mut bus, 0xC0);
        assert_eq!(cpu.registers.read_u8(Register8::B), 1);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::B, 0x81);
        run_cb(&mut cpu, &mut bus, 0x20);
        assert_eq!(cpu.registers.read_u8(Register8::B), 2);
        assert_eq!(flags(&cpu), (false, false, false, true));

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::B, 0x81);
        run_cb(&mut cpu, &mut bus, 0x28);
        assert_eq!(cpu.registers.read_u8(Register8::B), 0xC0);
        assert_eq!(flags(&cpu), (false, false, false, true));

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::B, 0x10);
        run_cb(&mut cpu, &mut bus, 0x30);
        assert_eq!(cpu.registers.read_u8(Register8::B), 1);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::B, 1);
        run_cb(&mut cpu, &mut bus, 0x38);
        assert_eq!(cpu.registers.read_u8(Register8::B), 0);
        assert_eq!(flags(&cpu), (true, false, false, true));
    }

    #[test]
    fn daa_decimal_add_and_subtract_adjustments() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0x3C);
        cpu.registers.set_flag(Flag::HalfCarry);
        run(&mut cpu, &mut bus, &[0x27]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0x42);
        assert_eq!(flags(&cpu), (false, false, false, false));

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0x9A);
        run(&mut cpu, &mut bus, &[0x27]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0);
        assert_eq!(flags(&cpu), (true, false, false, true));

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0x13);
        cpu.registers.set_flag(Flag::Substraction);
        cpu.registers.set_flag(Flag::HalfCarry);
        cpu.registers.set_flag(Flag::Carry);
        run(&mut cpu, &mut bus, &[0x27]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0xAD);
        assert_eq!(flags(&cpu), (false, true, false, true));
    }

    #[test]
    fn misc_flags_and_interrupt_enable_instructions() {
        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u8(Register8::A, 0x55);
        cpu.registers.set_flag(Flag::Carry);
        run(&mut cpu, &mut bus, &[0x2F]);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0xAA);
        assert_eq!(flags(&cpu), (false, true, true, true));

        let (mut cpu, mut bus) = setup();
        cpu.registers.set_flag(Flag::Zero);
        run(&mut cpu, &mut bus, &[0x37]);
        assert_eq!(flags(&cpu), (true, false, false, true));

        let (mut cpu, mut bus) = setup();
        cpu.ime = true;
        cpu.ime_enable_countdown = Some(1);
        run(&mut cpu, &mut bus, &[0xF3]);
        assert!(!cpu.ime);
        assert_eq!(cpu.ime_enable_countdown, None);

        let (mut cpu, mut bus) = setup();
        run(&mut cpu, &mut bus, &[0xFB]);
        assert!(!cpu.ime);
        assert_eq!(cpu.ime_enable_countdown, Some(1));
        run(&mut cpu, &mut bus, &[0x00]);
        assert!(!cpu.ime);
        run(&mut cpu, &mut bus, &[0x00]);
        assert!(cpu.ime);
    }

    #[test]
    fn illegal_opcodes_hang_cpu_and_prefix_decodes_next_byte() {
        let (mut cpu, mut bus) = setup();
        run(&mut cpu, &mut bus, &[0xD3]);
        assert!(cpu.hang);
        let pc = cpu.pc;
        cpu.clock(&mut bus);
        assert_eq!(cpu.pc, pc);

        let (mut cpu, mut bus) = setup();
        run_cb(&mut cpu, &mut bus, 0x7F);
        assert_eq!(cpu.registers.read_u8(Register8::A), 0);
        assert_eq!(flags(&cpu), (true, false, true, false));
        assert_cycles(&cpu, 2);
    }

    #[test]
    fn halt_and_reti_update_cpu_state() {
        let (mut cpu, mut bus) = setup();
        run(&mut cpu, &mut bus, &[0x76]);
        assert!(cpu.halt);
        assert_cycles(&cpu, 1);

        let (mut cpu, mut bus) = setup();
        cpu.registers.write_u16(Register16::SP, TEST_DATA);
        bus.write_u8(TEST_DATA, 0x34);
        bus.write_u8(TEST_DATA + 1, 0x12);
        run(&mut cpu, &mut bus, &[0xD9]);
        assert!(cpu.ime);
        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.registers.read_u16(Register16::SP), TEST_DATA + 2);
        assert_cycles(&cpu, 4);
    }
}
