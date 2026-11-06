use crate::{bus::Bus, cpu::instruction::{AddressingMode, AddressingMode16, Instruction}};

pub fn disassemble(bus: &Bus, pc: u16) -> (String, u16) {
    let instruction = get_instruction(bus, pc, false);
    let new_pc = pc.wrapping_add(instruction.length());
    let code = disassemble_instruction(bus, pc, instruction);
    (code, new_pc)
}

fn disassemble_instruction(bus: &Bus, pc: u16, instruction: Instruction) -> String {
    match instruction {
        Instruction::Nop => format!("NOP"),
        Instruction::Prefixed => {
            let pc = pc.wrapping_add(1);
            let instruction = get_instruction(bus, pc, true);
            disassemble_instruction(bus, pc, instruction)
        },
        Instruction::Illegal => format!("Illegal"),
        Instruction::Adc(addressing_mode) => 
            format!("ADC {}", disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::AddA(addressing_mode) => 
            format!("ADD A {}", disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::AddHL(register16) => format!("ADD HL {:?}", register16),
        Instruction::AddSP => format!("ADD SP {}", bus.read_u8(pc.wrapping_add(1)) as i8),
        Instruction::And(addressing_mode) => format!("AND {}", disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::Bit(offset, addressing_mode) => format!("BIT {} {}", offset, disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::Call(condition_code) => {
            let addr = fetch_u16(bus, pc);
            let cc_str = match condition_code {
                Some(cc) => format!("{:?}", cc),
                None => "".to_string(),
            };

            format!("CALL {} {:04X}", cc_str, addr)
        },
        Instruction::Ccf => format!("CCF"),
        Instruction::Cp(addressing_mode) => format!("CP {}", disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::Cpl => format!("CPL"),
        Instruction::Daa => format!("DAA"),
        Instruction::Dec8(register8) => format!("DEC {:?}", register8),
        Instruction::DecHL => format!("DEC [HL]"),
        Instruction::Dec16(register16) => format!("DEC {:?}", register16),
        Instruction::Di => format!("DI"),
        Instruction::Ei => format!("EI"),
        Instruction::Halt => format!("HALT"),
        Instruction::Inc8(register8) => format!("INC {:?}", register8),
        Instruction::IncHL => format!("INC [HL]"),
        Instruction::Inc16(register16) => format!("INC {:?}", register16),
        Instruction::Jp(condition_code) => {
            let addr = fetch_u16(bus, pc);
            let cc_str = match condition_code {
                Some(cc) => format!("{:?}", cc),
                None => "".to_string(),
            };

            format!("JP {} {:04X}", cc_str, addr)
        },
        Instruction::JpHL => format!("JP HL"),
        Instruction::Jr(condition_code) => {
            let addr = bus.read_u8(pc.wrapping_add(1)) as i8;
            let cc_str = match condition_code {
                Some(cc) => format!("{:?}", cc),
                None => "".to_string(),
            };

            format!("JR {} {}", cc_str, addr)
        },
        Instruction::Ld8(a, b) => 
            format!("LD {} {}", 
                disassemble_addressing_mode(bus, pc, a),
                disassemble_addressing_mode(bus, pc, b)
            ),
        Instruction::Ld16(register16, addressing_mode16) => 
            format!("LD {:?} {}", register16, disassemble_addressing_mode16(bus, pc, addressing_mode16)),
        Instruction::LdSp => format!("LD SP {}", fetch_u16(bus, pc.wrapping_add(1))),
        Instruction::LdHLSp => format!("LD HL SP,{}", bus.read_u8(pc.wrapping_add(1)) as i8),
        Instruction::Or(addressing_mode) => format!("OR {}", disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::Pop(register16) => format!("POP {:?}", register16),
        Instruction::Push(register16) => format!("PUSH {:?}", register16),
        Instruction::Res(offset, addressing_mode) => format!("RES {} {}", offset, disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::Ret(condition_code) => {
            let cc_str = match condition_code {
                Some(cc) => format!("{:?}", cc),
                None => "".to_string(),
            };

            format!("RET {}", cc_str)
        },
        Instruction::RetI => format!("RETI"),
        Instruction::Rla => format!("RLA"),
        Instruction::Rlca => format!("RLCA"),
        Instruction::Rl(addressing_mode) => format!("RL {}", disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::Rlc(addressing_mode) => format!("RLC {}", disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::Rra => format!("RRA"),
        Instruction::Rrca => format!("RRCA"),
        Instruction::Rr(addressing_mode) => format!("RR {}", disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::Rrc(addressing_mode) => format!("RRC {}", disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::Rst(vec) => format!("RST {}", vec),
        Instruction::Sbc(addressing_mode) => format!("SBC {}", disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::Scf => format!("SCF"),
        Instruction::Set(offset, addressing_mode) => format!("SET {} {}", offset, disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::Sla(addressing_mode) => format!("SLA {}", disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::Sra(addressing_mode) => format!("SRA {}", disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::Srl(addressing_mode) => format!("SRL {}", disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::Stop => format!("STOP {}", fetch_u16(bus, pc)),
        Instruction::Sub(addressing_mode) => format!("SUB {}", disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::Swap(addressing_mode) => format!("SWAP {}", disassemble_addressing_mode(bus, pc, addressing_mode)),
        Instruction::Xor(addressing_mode) => format!("XOR {}", disassemble_addressing_mode(bus, pc, addressing_mode)),
    }
}

fn disassemble_addressing_mode(bus: &Bus, pc: u16, addressing_mode: AddressingMode) -> String {
    match addressing_mode {
        AddressingMode::Register(register8) => format!("{:?}", register8),
        AddressingMode::Indirect(register16) => format!("[{:?}]", register16),
        AddressingMode::IdirectImm16 => format!("{:04X}", fetch_u16(bus, pc)),
        AddressingMode::IndirectInc(register16) => format!("[{:?}+]", register16),
        AddressingMode::IndirectDec(register16) => format!("[{:?}-]", register16),
        AddressingMode::IndirectImmH => {
            let addr = 0xFF00 | bus.read_u8(pc.wrapping_add(1)) as u16;
            let value = bus.read_u8(addr);
            format!("{:02X}", value)
        }
        AddressingMode::RegCIndirect => format!("[C]"),
        AddressingMode::Immediate8 => format!("{:02X}", bus.read_u8(pc.wrapping_add(1))),
    }
}

fn disassemble_addressing_mode16(bus: &Bus, pc: u16, addressing_mode: AddressingMode16) -> String {
    match addressing_mode {
        AddressingMode16::Register(register16) => format!("{:?}", register16),
        AddressingMode16::Immediate16 => format!("{:04X}", fetch_u16(bus, pc)),
    }
}

fn fetch_u16(bus: &Bus, pc: u16) -> u16 {
    let lo = bus.read_u8(pc.wrapping_add(1)) as u16;
    let hi = bus.read_u8(pc.wrapping_add(2)) as u16;

    (hi << 8) | lo
}

fn get_instruction(bus: &Bus, pc: u16, prefixed: bool) -> Instruction {
    let opcode = bus.read_u8(pc);
    Instruction::decode(opcode, prefixed)
}