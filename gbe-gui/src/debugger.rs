
use eframe::egui;
use gbe_core::Gameboy;

use crate::{
    assets::{PAUSE_SVG, PLAY_SVG, RESET_SVG, STEP_FRAME_SVG, STEP_INSTRUCTION_SVG}, debugger::{
        bus::{InterruptsComponent, JoypadComponent, TimersComponent}, components::{DebuggerComponentUi, components_ui}, cpu::{CpuDebuggerComponent, DisassemblerComponent}, ppu::{LCDComponent, WindowComponent},
    }, runner::GameboyRunner
};

pub(super) mod common;
mod cpu;
mod bus;
mod ppu;
mod components;

pub struct DebuggerUi {
    gameboy: Gameboy,
    debugger_comps_left: Vec<DebuggerComponentUi>,
    debugger_comps_right: Vec<DebuggerComponentUi>,

    gb_runner: GameboyRunner,
    is_running: bool,
}

impl DebuggerUi {
    pub fn new(ctx: &egui::Context, gameboy: Gameboy) -> Self {
        Self {
            gameboy,
            gb_runner: GameboyRunner::new(ctx),
            debugger_comps_left: vec![
                DebuggerComponentUi::new(CpuDebuggerComponent::new()),
                DebuggerComponentUi::new(DisassemblerComponent::new(10, 3)),
                DebuggerComponentUi::new(InterruptsComponent),
            ],
            debugger_comps_right: vec![
                DebuggerComponentUi::new(TimersComponent),
                DebuggerComponentUi::new(WindowComponent),
                DebuggerComponentUi::new(LCDComponent),
                DebuggerComponentUi::new(JoypadComponent)
            ],
            is_running: false,
        }
    }

    pub fn update(&mut self, ui: &mut egui::Ui) {
        if self.is_running {
            self.gb_runner.update(&mut self.gameboy, ui.ctx());
            ui.request_repaint_after(self.gb_runner.frame_duration());
        }
        self.ui(ui);
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let side_panels_width = ui.available_width() / 4.0;
        let gameboy_snapshot = self.gameboy.get_snapshot();

        egui::Panel::left("Execution State")
            .resizable(false)
            .exact_size(side_panels_width)
            .show(ui, |ui| {
                components_ui(ui, &mut self.debugger_comps_left, &gameboy_snapshot);
            });

        egui::Panel::right("Right Panel")
            .resizable(false)
            .exact_size(side_panels_width)
            .show(ui, |ui| {
                components_ui(ui, &mut self.debugger_comps_right, &gameboy_snapshot);
            });

        egui::CentralPanel::default().show(ui, |ui| {
            self.draw_emulation_control(ui);
            self.draw_gameboy_screen(ui);
        });
    }

    fn draw_gameboy_screen(&mut self, ui: &mut egui::Ui) {
        if self.is_running {
            self.gb_runner.ui(ui);
        }
        else {
            self.gb_runner.ui_with_framebuffer(ui, self.gameboy.get_framebuffer());
        }
    }

    fn draw_emulation_control(&mut self, ui: &mut egui::Ui) {

        let icon_size= egui::vec2(50.0, 50.0);
        
        ui.add_space(10.0);

        let spacing = ui.spacing().item_spacing.x;
        let btn_pad = ui.spacing().button_padding.x;
        let n = if self.is_running { 2.0 } else { 4.0 };
        let total_size = spacing * (n - 1.0) + (icon_size.x + btn_pad * 2.0) * n;
        let pad = ((ui.available_width() - total_size) * 0.5 - spacing).max(0.0);

        ui.horizontal(|ui| {
            ui.add_space(pad);

            if self.is_running {
                let clicked = common::icon(ui, PAUSE_SVG, "pause", icon_size).clicked();
                self.is_running = !clicked;
            }
            else {
                let play_clicked = common::icon(ui, PLAY_SVG, "play", icon_size).clicked();
                let step_instr_clicked = common::icon(ui, STEP_INSTRUCTION_SVG, "step", icon_size).clicked();
                let step_frame_clicked = common::icon(ui, STEP_FRAME_SVG, "frame", icon_size).clicked();

                self.is_running = play_clicked;
                if step_instr_clicked { self.gameboy.next_instruction(); }
                if step_frame_clicked { self.gameboy.next_frame(); }
            }

            let reset_clicked = common::icon(ui, RESET_SVG, "reset", icon_size).clicked();
            if reset_clicked { 
                self.gameboy.reset();
                self.is_running = false;
            }
        });
    }
}
