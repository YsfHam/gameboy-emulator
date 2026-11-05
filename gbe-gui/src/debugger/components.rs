use eframe::egui;
use gbe_core::GameboySnapshot;

use crate::debugger::common::{self, PANEL_BG};

pub trait DebuggerComponent {
    fn name(&self) -> &'static str;
    fn ui(&mut self, ui: &mut egui::Ui, gameboy_snapshot: &GameboySnapshot);
}

pub struct DebuggerComponentUi {
    comp: Box<dyn DebuggerComponent>,
}

impl DebuggerComponentUi {
    pub fn new(comp: impl DebuggerComponent + 'static) -> Self {
        Self {
            comp: Box::new(comp)
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, gameboy_snapshot: &GameboySnapshot) {
        common::component_section(ui, self.comp.name(), |ui| {
            self.comp.ui(ui, gameboy_snapshot);
        });
    }
}

pub fn components_ui(ui: &mut egui::Ui, comps_ui: &mut [DebuggerComponentUi], gameboy_snapshot: &GameboySnapshot) {
    egui::Frame::new()
    .fill(PANEL_BG)
    .inner_margin(12.0)
    .show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for comp in comps_ui {
                comp.ui(ui, &gameboy_snapshot);
            }
        });
    });
}