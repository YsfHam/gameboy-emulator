use std::fs;

use eframe::egui::{self, ViewportBuilder};
use gbe_core::{Cartridge, Gameboy};

use crate::{args::CLIArgs, debugger::DebuggerUi};

mod render;
mod args;
mod debugger;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
        .with_maximized(true),
        ..Default::default()
    };
    eframe::run_native(
        "GB Emulator",
        native_options, 
        Box::new(|cc| {
            Ok(GBEmulatorApp::new(cc)
            .map(Box::new)
            .unwrap())
        })
    )
}

struct GBEmulatorApp {
    debugger_ui: DebuggerUi
}

impl GBEmulatorApp {
    fn new(_: &eframe::CreationContext<'_>) -> Result<Self, String> {
        let args = CLIArgs::parse_args().ok_or("Args are required")?;
        let cart = Cartridge::from_memory(fs::read(args.rom_path).map_err(|err| err.to_string())?);
        let gameboy =  Gameboy::new(cart);
        Ok(Self {
            debugger_ui: DebuggerUi::new(gameboy)
        })
    }
}

impl eframe::App for GBEmulatorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        self.debugger_ui.update(ui);
    }
}