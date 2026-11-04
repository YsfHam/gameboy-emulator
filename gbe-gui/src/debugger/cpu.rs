use gbe_core::{CpuSnapshot, disassemble};

use crate::debugger::DebuggerComponent;

pub struct CpuDebuggerComponent {
    last_spanshot: Option<CpuSnapshot>,   
}

impl CpuDebuggerComponent {
    pub fn new() -> Self {
        Self {
            last_spanshot: None
        }
    }
}

impl DebuggerComponent for CpuDebuggerComponent {
    fn name(&self) -> &str {
        "CPU"
    }

    fn ui(&mut self, ui: &mut eframe::egui::Ui, gameboy_snapshot: &gbe_core::GameboySnapshot) {
        let cpu_snapshot = gameboy_snapshot.cpu;
        ui.label(format!("AF\t{:04X}", cpu_snapshot.reg_af));
        ui.label(format!("BC\t{:04X}", cpu_snapshot.reg_bc));
        ui.label(format!("DE\t{:04X}", cpu_snapshot.reg_de));
        ui.label(format!("HL\t{:04X}", cpu_snapshot.reg_hl));
        ui.label(format!("SP\t{:04X}", cpu_snapshot.sp));
        ui.label(format!("PC\t{:04X}", cpu_snapshot.pc));
        ui.label(format!("Flags: Z N H C IME {}", cpu_snapshot.ime as u8));
        self.last_spanshot = Some(cpu_snapshot);
    }
}

pub struct DisassemblerComponent {
    max_elements: u32,
    min_forward: u32,
    old_pc: i32
}

impl DisassemblerComponent {
    pub fn new(max_elements: u32, min_forward: u32) -> Self {
        Self {
            max_elements,
            min_forward,
            old_pc: 0
        }
    }
}

impl DebuggerComponent for DisassemblerComponent {
    fn name(&self) -> &str {
        "Disassembly"
    }

    fn ui(&mut self, ui: &mut eframe::egui::Ui, gameboy_snapshot: &gbe_core::GameboySnapshot) {
        let pc = gameboy_snapshot.cpu.pc as i32;
        let diff = (self.old_pc - pc).abs();
        self.old_pc = pc;

        let max_diff = self.max_elements as i32 - self.min_forward as i32 - 1;
        let base = (pc - diff.min(max_diff)) as u32;
        let range = base..(base+self.max_elements);
        for i in range.map(|x| x as u16) {
            let disassembly_line = disassemble::disassemble(gameboy_snapshot.bus, i);
            if i == gameboy_snapshot.cpu.pc {
                ui.label(format!("here {}", disassembly_line));
            }
            else {
                ui.label(format!("{}", disassembly_line));
            }
        }
    }
}