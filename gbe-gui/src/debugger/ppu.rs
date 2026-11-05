use eframe::egui;
use gbe_core::LCDCFlags;

use crate::debugger::{common::{self, MUTED_TEXT}, components::DebuggerComponent};

pub struct WindowComponent;

impl DebuggerComponent for WindowComponent {
    fn name(&self) -> &'static str {
        "WINDOW"
    }

    fn ui(&mut self, ui: &mut eframe::egui::Ui, gameboy_snapshot: &gbe_core::GameboySnapshot) {
        let window = &gameboy_snapshot.bus.window;

        egui::Grid::new("ppu_window")
        .spacing(egui::vec2(8.0, 8.0))
        .show(ui, |ui| {
            common::cell(ui, "WX", &format!("{:02X}", window.wx));
            common::cell(ui, "WY", &format!("{:02X}", window.wy));
        });
    }
}

pub struct LCDComponent;

impl DebuggerComponent for LCDComponent {
    fn name(&self) -> &'static str {
        "LCD"
    }

    fn ui(&mut self, ui: &mut egui::Ui, gameboy_snapshot: &gbe_core::GameboySnapshot) {
        let lcd = &gameboy_snapshot.bus.lcd;

        egui::Grid::new("lcd")
        .spacing(egui::vec2(8.0, 8.0))
        .show(ui, |ui| {
            common::cell(ui, "LCDC", &format!("{:02X}", lcd.control));
            ui.end_row();
            common::cell(ui, "YL", &format!("{:02X}", lcd.ly()));
            common::cell(ui, "LYC", &format!("{:02X}", lcd.lyc()));
            ui.end_row();
            common::cell(ui, "SCX", &format!("{:02X}", lcd.scx));
            common::cell(ui, "SCY", &format!("{:02X}", lcd.scy));
            ui.end_row();
            common::cell(ui, "BGP", &format!("{:02X}", lcd.bgp));
            common::cell(ui, "OBP0", &format!("{:02X}", lcd.obp0));
            common::cell(ui, "OBP1", &format!("{:02X}", lcd.obp1));
        });

        ui.add_space(8.0);
        ui.label(
            eframe::egui::RichText::new("LCDC FLAGS")
                .small()
                .strong()
                .color(MUTED_TEXT),
        );

        let flags = [
            LCDCFlags::BGWindowEnable,
            LCDCFlags::ObjEnable,
            LCDCFlags::ObjSize,
            LCDCFlags::BGTileMap,
            LCDCFlags::BGWindowTiles,
            LCDCFlags::WindowEnable,
            LCDCFlags::WindowTileMap,
            LCDCFlags::LCDPpuEnable
        ];

        egui::Grid::new("lcdc_state")
        .spacing(egui::vec2(8.0, 8.0))
        .show(ui, |ui| {
            for flags_window in flags.windows(2) {
                common::state_chip(
                    ui, 
                    &format!("{:?}", flags_window[0]),
                    lcd.get_lcdc_flag(flags_window[0])
                );
                common::state_chip(
                    ui, 
                    &format!("{:?}", flags_window[1]),
                    lcd.get_lcdc_flag(flags_window[1])
                );
                ui.end_row();
            }
        });
    }
}