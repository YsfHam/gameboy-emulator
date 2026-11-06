use eframe::egui;
use gbe_core::{Interrupt, joypad::JoypButton};

use crate::debugger::{common, components::DebuggerComponent};

pub struct InterruptsComponent;

impl DebuggerComponent for InterruptsComponent {
    fn name(&self) -> &'static str {
        "INTERRUPTS"
    }

    fn ui(&mut self, ui: &mut eframe::egui::Ui, gameboy_snapshot: &gbe_core::GameboySnapshot) {
        let interrupts = &gameboy_snapshot.bus.interrupts_flags;
        eframe::egui::Grid::new("interrupt_registers")
            .num_columns(4)
            .spacing(eframe::egui::vec2(8.0, 7.0))
            .show(ui, |ui| {
                common::cell(ui, "IF", &format!("{:02X}", interrupts.interrupt_requested));
                common::cell(ui, "IE", &format!("{:02X}", interrupts.interrupt_enabled));
                ui.end_row();
            });

        ui.add_space(4.0);
        eframe::egui::Grid::new("interrupt_status")
            .num_columns(4)
            .striped(true)
            .spacing(eframe::egui::vec2(8.0, 7.0))
            .show(ui, |ui| {
                for interrupt in [
                    Interrupt::VBlank,
                    Interrupt::LCD,
                    Interrupt::Timer,
                    Interrupt::Serial,
                    Interrupt::Joypad,
                ] {
                    ui.label(format!("{:?}", interrupt));
                    common::state_chip(ui, "EN", interrupts.is_interrupt_enabled(interrupt));
                    common::state_chip(ui, "REQ", interrupts.is_interrupt_requested(interrupt));
                    common::state_chip(ui, "PEN", interrupts.is_interrupt_pending(interrupt));
                    ui.end_row();
                }
            });
    }
}

pub struct TimersComponent;

impl DebuggerComponent for TimersComponent {
    fn name(&self) -> &'static str {
        "TIMERS"
    }

    fn ui(&mut self, ui: &mut eframe::egui::Ui, gameboy_snapshot: &gbe_core::GameboySnapshot) {
        let timers = &gameboy_snapshot.bus.timers;
        let div = gameboy_snapshot.bus.div_register;
        eframe::egui::Grid::new("bus_timers")
        .num_columns(4)
        .spacing(egui::vec2(8.0, 8.0))
        .show(ui, |ui| {
            common::cell(ui, "DIV", &format!("{:04X}", div));
            common::cell(ui, "TAC", &format!("{:02X}", timers.tac));
            ui.end_row();
            common::cell(ui, "TMA", &format!("{:02X}", timers.tma));
            common::cell(ui, "TIMA", &format!("{:02X}", timers.tima));
        });
    }
}

pub struct JoypadComponent;

impl DebuggerComponent for JoypadComponent {
    fn name(&self) -> &'static str {
        "JOYPAD"
    }

    fn ui(&mut self, ui: &mut egui::Ui, gameboy_snapshot: &gbe_core::GameboySnapshot) {
        let buttons = [
            [JoypButton::A,
            JoypButton::B,
            JoypButton::Select,
            JoypButton::Start,],
            [JoypButton::Right,
            JoypButton::Left,
            JoypButton::Up,
            JoypButton::Down,]
        ];

        egui::Grid::new("joypad")
        .spacing(egui::vec2(8.0, 8.0))
        .show(ui, |ui| {
            for btns_set in buttons {
                for btn in btns_set {
                    common::state_chip(ui, &format!("{btn:?}"), gameboy_snapshot.bus.joypad.is_button_pressed(btn));
                }
                ui.end_row();
            }

            common::state_chip(ui, "D-pad", gameboy_snapshot.bus.joypad.select_d_pad());
            common::state_chip(ui, "Button", gameboy_snapshot.bus.joypad.select_button());
        });
    }
}