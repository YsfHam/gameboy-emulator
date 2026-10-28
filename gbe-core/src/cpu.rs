use crate::{bus::Bus, cpu::{instruction::{AddressingMode, AddressingMode16, Instruction}, registers::{Flag, Register8, Register16, Registers}}};

pub mod registers;
pub mod instruction;
mod helpers;

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
        Self {
            registers: Registers::new(),
            pc: 0,
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

        if self.hang { return; }

        if self.remaining_cycles == 0 {
            self.handle_interrupts();
            if self.halt {
                return;
            }
            let instruction = self.fetch_instruction(bus);
            self.execute(bus, instruction);
        }
        self.remaining_cycles = self.remaining_cycles.saturating_sub(1);
    }

    fn handle_interrupts(&mut self) {
        self.set_ime();
        let interrupt_occured = false;
        self.halt &= !interrupt_occured;
        self.halt_bug = self.halt && !self.ime && interrupt_occured;
        if self.ime && interrupt_occured {
            todo!("Run interrupts");
        }
    }

    fn set_ime(&mut self) {
        match self.ime_enable_countdown {
            Some(0) => {
                self.ime = true;
                self.ime_enable_countdown = None;
            },
            Some(n) => self.ime_enable_countdown = Some(n - 1),
            None => (),
        }
    }
}


#[derive(Debug, Copy, Clone)]
pub enum ConditionCode {
    Z, NZ, C, NC
}


impl Cpu {
    fn fetch_u8(&mut self, bus: &Bus) -> u8 {
        let data = bus.read_u8(self.pc);
        if !self.halt_bug {
            self.pc = self.pc.wrapping_add(1);
        }
        else {
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
            },
            AddressingMode::Immediate8 => self.fetch_u8(bus),
            AddressingMode::IdirectImm16 => {
                let addr = self.fetch_u16(bus);
                bus.read_u8(addr)
            },
            AddressingMode::IndirectInc(register16) => {
                let addr = self.registers.read_u16(register16);
                self.registers.write_u16(register16, addr.wrapping_add(1));
                bus.read_u8(addr)
            },
            AddressingMode::IndirectDec(register16) => {
                let addr = self.registers.read_u16(register16);
                self.registers.write_u16(register16, addr.wrapping_sub(1));
                bus.read_u8(addr)
            },
            AddressingMode::IndirectImmH => {
                let addr = 0xFF00 | self.fetch_u8(bus) as u16;
                bus.read_u8(addr)
            },
            AddressingMode::CarryH => {
                let addr = 0xFF00 | self.registers.get_flag(Flag::Carry) as u16;
                bus.read_u8(addr)
            },
        }
    }

    fn addressing_mode_write(&mut self, bus: &mut Bus, mode: AddressingMode, value: u8) {
        match mode {
            AddressingMode::Register(register8) => self.registers.write_u8(register8, value),
            AddressingMode::Indirect(register16) => {
                let addr = self.registers.read_u16(register16);
                bus.write_u8(addr, value);
            },
            AddressingMode::IdirectImm16 => {
                let addr = self.fetch_u16(bus);
                bus.write_u8(addr, value);
            },
            AddressingMode::IndirectInc(register16) => {
                let addr = self.registers.read_u16(register16);
                self.registers.write_u16(register16, addr.wrapping_add(1));
                bus.write_u8(addr, value);
            },
            AddressingMode::IndirectDec(register16) => {
                let addr = self.registers.read_u16(register16);
                self.registers.write_u16(register16, addr.wrapping_sub(1));
                bus.write_u8(addr, value);
            },
            AddressingMode::IndirectImmH => {
                let addr = 0xFF00 | self.fetch_u8(bus) as u16;
                bus.write_u8(addr, value);
            },
            AddressingMode::CarryH => {
                let addr = 0xFF00 | self.registers.get_flag(Flag::Carry) as u16;
                bus.write_u8(addr, value);
            },
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
            => 1,
            AddressingMode::IdirectImm16 => 3,
            AddressingMode::IndirectImmH => 2,
            AddressingMode::CarryH => 1,
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
    flags_update: FlagsUpdate
}

impl Cpu {
    fn execute(&mut self, bus: &mut Bus, instruction: Instruction) {
        let result = match instruction {
            Instruction::Nop => self.nop(),

            Instruction::Adc(addressing_mode) => {
                let cycles = Self::alu_cycles(&addressing_mode);
                let value = self.addressing_mode_read(bus, addressing_mode);
                self.adc(value, cycles)
            },
            Instruction::AddA(addressing_mode) => {
                let cycles = Self::alu_cycles(&addressing_mode);
                let value = self.addressing_mode_read(bus, addressing_mode);
                self.add_a(value, cycles)
            },
            Instruction::AddHL(register16) => {
                let value = self.registers.read_u16(register16);
                self.add_hl(value)
            },
            Instruction::AddSP => {
                let value = self.fetch_u8(bus);
                self.add_sp(value as i8)
            },
            Instruction::And(addressing_mode) => {
                let cycles = Self::alu_cycles(&addressing_mode);
                let value = self.addressing_mode_read(bus, addressing_mode);
                self.and(value, cycles)
            },
            Instruction::Bit(offset, addressing_mode) => {
                let value = self.addressing_mode_read(bus, addressing_mode);
                let alu_cycles = Self::alu_cycles(&addressing_mode);
                self.bit(offset, value, alu_cycles)
            },
            Instruction::Call(condition_code) => {
                let addr = self.fetch_u16(bus);
                match condition_code {
                    Some(cc) => self.call_cc(bus, cc, addr),
                    None => self.call(bus, addr)
                }
            }
            Instruction::Ccf => self.ccf(),
            Instruction::Cp(addressing_mode) => {
                let value = self.addressing_mode_read(bus, addressing_mode);
                let alu_cycles = Self::alu_cycles(&addressing_mode);
                self.cp(value,alu_cycles)
            },
            Instruction::Cpl => self.cpl(),
            Instruction::Daa => self.daa(),
            Instruction::Dec8(register8) => self.dec_r8(register8),
            Instruction::DecHL => self.dec_hl(bus),
            Instruction::Dec16(register16) => self.dec_r16(register16),
            Instruction::Di => self.di(),
            Instruction::Ei => self.ei(),
            Instruction::Halt => self.halt(),
            Instruction::Inc8(register8) => self.inc_r8(register8),
            Instruction::IncHL => self.inc_hl(bus),
            Instruction::Inc16(register16) => self.inc_r16(register16),
            Instruction::Jp(condition_code) => {
                let addr = self.fetch_u16(bus);
                match condition_code {
                    Some(cc) => self.jp_cc(cc, addr),
                    None => self.jp(addr),
                }
            },
            Instruction::JpHL => self.jp_hl(),
            Instruction::Jr(condition_code) => {
                let addr = self.fetch_u8(bus);
                match condition_code {
                    Some(cc) => self.jr_cc(cc, addr),
                    None => self.jr(addr),
                }
            },
            Instruction::Ld8(dest, src) => self.ld(bus, dest, src),
            Instruction::Ld16(register16, addressing_mode16) => {
                let value = self.addressing_mode16_read(bus, addressing_mode16);
                self.ld_r16(register16, value)
            }
            Instruction::LdSp => {
                let addr = self.fetch_u16(bus);
                self.ld_sp(bus, addr)
            },
            Instruction::LdHLSp => {
                let value = self.fetch_u8(bus);
                self.ld_hl_sp(value as i8)
            },
            Instruction::Or(addressing_mode) => {
                let value = self.addressing_mode_read(bus, addressing_mode);
                let alu_cycles = Self::alu_cycles(&addressing_mode);
                self.or(value, alu_cycles)
            },
            Instruction::Pop(register16) => {
                self.pop(bus, register16)
            },
            Instruction::Push(register16) => {
                self.push(bus, register16)
            },
            Instruction::Res(bit, addressing_mode) => {
                self.res(bus, bit, addressing_mode)
            },
            Instruction::Ret(condition_code) => {
                match condition_code {
                    Some(cc) => self.ret_cc(bus, cc),
                    None => self.ret(bus),
                }
            },
            Instruction::RetI => self.reti(bus),
            Instruction::Rl(addressing_mode) => self.rl(bus, addressing_mode),
            Instruction::Rlc(addressing_mode) => self.rlc(bus, addressing_mode),
            Instruction::Rr(addressing_mode) => self.rr(bus, addressing_mode),
            Instruction::Rrc(addressing_mode) => self.rrc(bus, addressing_mode),
            Instruction::Rst(vec) => self.rst(bus, vec),
            Instruction::Sbc(addressing_mode) => {
                let value = self.addressing_mode_read(bus, addressing_mode);
                let alu_cycles = Self::alu_cycles(&addressing_mode);
                self.sbc(value, alu_cycles)
            },
            Instruction::Scf => self.scf(),
            Instruction::Set(offset, addressing_mode) => {
                self.set(bus, offset, addressing_mode)
            },
            Instruction::Sla(addressing_mode) => self.sla(bus, addressing_mode),
            Instruction::Sra(addressing_mode) => self.sra(bus, addressing_mode),
            Instruction::Srl(addressing_mode) => self.srl(bus, addressing_mode),
            Instruction::Stop => self.stop(bus),
            Instruction::Sub(addressing_mode) => {
                let value = self.addressing_mode_read(bus, addressing_mode);
                let alu_cycles = Self::alu_cycles(&addressing_mode);
                self.sub(value, alu_cycles)
            },
            Instruction::Swap(addressing_mode) => self.swap(bus, addressing_mode),
            Instruction::Xor(addressing_mode) => {
                let value = self.addressing_mode_read(bus, addressing_mode);
                let alu_cycles = Self::alu_cycles(&addressing_mode);
                self.xor(value, alu_cycles)
            },
            Instruction::Prefixed => {
                self.prefixed_instruction = true;
                ExecutionResult {
                    machine_cycles: 1,
                    flags_update: Default::default()
                }
            },
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
        }
        else {
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
        }
        else {
            let mut adjustement = 0;
            let mut carry_flag_set= false;
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

    fn halt(&mut self) -> ExecutionResult {
        self.halt = true;
        ExecutionResult {
            machine_cycles: 1,
            flags_update: Default::default()
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
            machine_cycles: 1,
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
        }
        else {
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
            flags_update: Default::default()
        }
    }

    fn jr(&mut self, rel_addr: u8) -> ExecutionResult {
        self.pc = self.pc.wrapping_add_signed(rel_addr as i16);
        ExecutionResult {
            machine_cycles: 3,
            flags_update: Default::default(),
        }
    }

    fn jr_cc(&mut self, cc: ConditionCode, rel_addr: u8) -> ExecutionResult {
        if self.check_condition(cc) {
            self.jr(rel_addr)
        }
        else {
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
            flags_update: Default::default()
        }
    }

    fn ld_r16(&mut self, reg: Register16, value: u16) -> ExecutionResult {
        self.registers.write_u16(reg, value);
        ExecutionResult {
            machine_cycles: 3,
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

        let half_carry = (sp & 0xF) as i8 + (value & 0xF) as i8 > 0xF;
        let carry = (sp & 0xFF) as i16 + (value as i16) > 0xFF;

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
                flag_h: Some(true),
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


    fn res(&mut self, bus: &mut Bus, offset: u8, addressing_mode: AddressingMode) -> ExecutionResult {
        let value = self.addressing_mode_read(bus, addressing_mode);
        let result = value & !(1 << offset);
        self.addressing_mode_write(bus, addressing_mode, result);
        ExecutionResult {
            machine_cycles: 2 + Self::alu_cycles(&addressing_mode) * 2,
            flags_update: Default::default()
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
        }
        else {
            ExecutionResult {
                machine_cycles: 4,
                flags_update: Default::default(),
            }
        }
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
            machine_cycles: if addressing_mode == AddressingMode::Register(Register8::A) { 1 } else { 2 },
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0 && addressing_mode != AddressingMode::Register(Register8::A)),
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
            machine_cycles: if addressing_mode == AddressingMode::Register(Register8::A) { 1 } else { 2 },
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0 && addressing_mode != AddressingMode::Register(Register8::A)),
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
            machine_cycles: if addressing_mode == AddressingMode::Register(Register8::A) { 1 } else { 2 },
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0 && addressing_mode != AddressingMode::Register(Register8::A)),
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
            machine_cycles: if addressing_mode == AddressingMode::Register(Register8::A) { 1 } else { 2 },
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0 && addressing_mode != AddressingMode::Register(Register8::A)),
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
            flags_update: Default::default()
        }
    }

    fn sbc(&mut self, value: u8, alu_cycles: u8) -> ExecutionResult {
        let reg_a = self.registers.read_u8(Register8::A);
        let (result, half_carry, carry) = 
            helpers::carrying_sub_u8(reg_a, value, self.registers.get_flag(Flag::Carry));
        ExecutionResult {
            machine_cycles: alu_cycles + 1,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(true),
                flag_h: Some(half_carry),
                flag_c: Some(carry)
            }
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
            flags_update: Default::default()
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
                flag_c: Some(carry)
            }
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
                flag_c: Some(carry)
            }
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
                flag_c: Some(carry)
            }
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
        let result= helpers::swap_nibbles(value);
        self.addressing_mode_write(bus, addressing_mode, result);

        ExecutionResult {
            machine_cycles: 2 + Self::alu_cycles(&addressing_mode) * 2,
            flags_update: FlagsUpdate {
                flag_z: Some(result == 0),
                flag_n: Some(false),
                flag_h: Some(false),
                flag_c: Some(false)
            }
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
            }
        }
    }

}