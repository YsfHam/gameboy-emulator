use std::collections::VecDeque;

use gbe_core::{Bus, disassemble::disassemble};

use crate::debugger::{
    common::{self, ACCENT, ACTIVE_BG, CARD_BG, MUTED_TEXT}, components::DebuggerComponent,
};

pub struct CpuDebuggerComponent;

impl CpuDebuggerComponent {
    pub fn new() -> Self {
        Self
    }
}

impl DebuggerComponent for CpuDebuggerComponent {
    fn name(&self) -> &'static str {
        "CPU"
    }

    fn ui(&mut self, ui: &mut eframe::egui::Ui, gameboy_snapshot: &gbe_core::GameboySnapshot) {
        let cpu_snapshot = gameboy_snapshot.cpu;
        eframe::egui::Grid::new("cpu_registers")
            .num_columns(4)
            .spacing(eframe::egui::vec2(8.0, 7.0))
            .show(ui, |ui| {
                common::cell(ui, "AF", &format!("{:04X}", cpu_snapshot.reg_af));
                common::cell(ui, "BC", &format!("{:04X}", cpu_snapshot.reg_bc));
                ui.end_row();
                common::cell(ui, "DE", &format!("{:04X}", cpu_snapshot.reg_de));
                common::cell(ui, "HL", &format!("{:04X}", cpu_snapshot.reg_hl));
                ui.end_row();
                common::cell(ui, "SP", &format!("{:04X}", cpu_snapshot.sp));
                common::cell(ui, "PC", &format!("{:04X}", cpu_snapshot.pc));
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.label(
            eframe::egui::RichText::new("CPU STATE")
                .small()
                .strong()
                .color(MUTED_TEXT),
        );
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            common::state_chip(ui, "Z", cpu_snapshot.z_flag);
            common::state_chip(ui, "N", cpu_snapshot.n_flag);
            common::state_chip(ui, "H", cpu_snapshot.h_flag);
            common::state_chip(ui, "C", cpu_snapshot.c_flag);
            common::state_chip(ui, "IME", cpu_snapshot.ime);
        });
    }
}

pub struct DisassemblerComponent {
    max_elements: u32,
    min_forward: u32,
    instructions: VecDeque<(u16, String)>,
    last_pc: Option<u16>,
}

impl DisassemblerComponent {
    pub fn new(max_elements: u32, min_forward: u32) -> Self {
        Self {
            max_elements,
            min_forward,
            instructions: VecDeque::new(),
            last_pc: None,
        }
    }

    fn flush_instruction(&mut self, pc: u16, bus: &Bus) {
        self.instructions.clear();
        let (instruction_code, new_pc) = disassemble(&bus, pc);
        self.instructions.push_back((pc, instruction_code));
        self.last_pc = Some(new_pc);
        self.extend_instructions(&bus, self.max_elements - 1);
    }

    fn extend_instructions(&mut self, bus: &Bus, extended_size: u32) {
        let mut pc = match self.last_pc {
            Some(pc) => pc,
            None => return
        };
        for _ in 0..extended_size {
            let (instruction_code, new_pc) = disassemble(&bus, pc);
            self.instructions.push_back((pc, instruction_code));
            pc = new_pc;
        }
        self.last_pc = Some(pc);
        if self.instructions.len() > self.max_elements as usize {
            for _ in 0..extended_size {
                self.instructions.pop_front();
            }
        }
    }

    fn find_pc(&self, pc: u16) -> Option<usize> {
        self.instructions
        .iter()
        .enumerate()
        .find(|(_, (instr_pc, _))| pc == *instr_pc)
        .map(|(index, _)| index)
    }
}

impl DebuggerComponent for DisassemblerComponent {
    fn name(&self) -> &'static str {
        "DISASSEMBLY"
    }

    fn ui(&mut self, ui: &mut eframe::egui::Ui, gameboy_snapshot: &gbe_core::GameboySnapshot) {
        let pc = gameboy_snapshot.cpu.pc;
        let bus = gameboy_snapshot.bus;
        let pc_index = self.find_pc(pc);
        if self.instructions.is_empty() || pc_index.is_none() {
            self.flush_instruction(pc, bus);
        }
        let extension_threshold = self
            .max_elements
            .saturating_sub(self.min_forward);
        if let Some(index) = pc_index {
            if index >= extension_threshold as usize {
                self.extend_instructions(bus, self.min_forward);
            }
        }

        eframe::egui::ScrollArea::vertical()
            .max_height(280.0)
            .show(ui, |ui| {
                for (address, instruction) in &self.instructions {
                    let is_current = *address == pc;

                    eframe::egui::Frame::new()
                        .fill(if is_current { ACTIVE_BG } else { CARD_BG })
                        .corner_radius(4.0)
                        .inner_margin(eframe::egui::Margin::symmetric(6, 4))
                        .show(ui, |ui| {
                            let row_width = (ui.available_width() - 12.0).max(0.0);
                            ui.set_width(row_width);
                            ui.horizontal(|ui| {
                                ui.label(
                                    eframe::egui::RichText::new(format!("{address:04X}"))
                                        .small()
                                        .monospace()
                                        .color(if is_current { ACCENT } else { MUTED_TEXT }),
                                );
                                if is_current {
                                    eframe::egui::Frame::new()
                                        .fill(common::ACCENT_DIM)
                                        .corner_radius(3.0)
                                        .inner_margin(eframe::egui::Margin::symmetric(4, 2))
                                        .show(ui, |ui| {
                                            ui.label(
                                                eframe::egui::RichText::new("PC")
                                                    .small()
                                                    .strong()
                                                    .color(ACCENT),
                                            );
                                        });
                                }
                                ui.add_space(2.0);

                                if let Some((mnemonic, operands)) = instruction.split_once(' ') {
                                    ui.label(
                                        eframe::egui::RichText::new(mnemonic)
                                            .monospace()
                                            .strong()
                                            .color(ACCENT),
                                    );
                                    ui.label(
                                        eframe::egui::RichText::new(operands)
                                            .monospace()
                                            .color(common::TEXT),
                                    );
                                } else {
                                    ui.label(
                                        eframe::egui::RichText::new(instruction)
                                            .monospace()
                                            .strong()
                                            .color(if is_current { ACCENT } else { common::TEXT }),
                                    );
                                }
                            });
                        });
                }
            });
    }
}
