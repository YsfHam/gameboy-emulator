use crate::cpu::{Cpu, registers::{Flag, Register16}};

#[derive(Clone, Debug, Copy)]
pub struct CpuSnapshot {
    pub reg_af: u16,
    pub reg_bc: u16,
    pub reg_de: u16,
    pub reg_hl: u16,
    pub sp: u16,
    pub pc: u16,
    pub ime: bool,
    pub z_flag: bool,
    pub n_flag: bool,
    pub h_flag: bool,
    pub c_flag: bool,
}

impl From<&Cpu> for CpuSnapshot {
    fn from(cpu: &Cpu) -> Self {
        Self {
            reg_af: cpu.registers.read_u16(Register16::AF),
            reg_bc: cpu.registers.read_u16(Register16::BC),
            reg_de: cpu.registers.read_u16(Register16::DE),
            reg_hl: cpu.registers.read_u16(Register16::HL),
            sp: cpu.registers.read_u16(Register16::SP),
            pc: cpu.pc,
            ime: cpu.ime,
            z_flag: cpu.registers.get_flag(Flag::Zero),
            n_flag: cpu.registers.get_flag(Flag::Substraction),
            h_flag: cpu.registers.get_flag(Flag::HalfCarry),
            c_flag: cpu.registers.get_flag(Flag::Carry),
        }
    }
}