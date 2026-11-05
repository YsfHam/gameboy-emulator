use std::time::{Duration, Instant};

use eframe::egui;
use gbe_core::{Gameboy, GameboySnapshot};

use crate::{
    debugger::{
        common::PANEL_BG,
        cpu::{CpuDebuggerComponent, DisassemblerComponent},
    },
    render::GameBoyRenderer,
};

pub(super) mod common;
mod cpu;

trait DebuggerComponent {
    fn name(&self) -> &'static str;
    fn ui(&mut self, ui: &mut egui::Ui, gameboy_snapshot: &GameboySnapshot);
}

struct DebuggerComponentUi {
    comp: Box<dyn DebuggerComponent>,
}

impl DebuggerComponentUi {
    fn ui(&mut self, ui: &mut egui::Ui, gameboy_snapshot: &GameboySnapshot) {
        common::component_section(ui, self.comp.name(), |ui| {
            self.comp.ui(ui, gameboy_snapshot);
        });
    }
}

pub struct DebuggerUi {
    gameboy: Gameboy,
    gb_renderer: GameBoyRenderer,
    debugger_comps: Vec<DebuggerComponentUi>,

    last_clock: Instant,
    refresh_rate: Duration,
}

impl DebuggerUi {
    pub fn new(gameboy: Gameboy) -> Self {
        Self {
            gameboy,
            gb_renderer: GameBoyRenderer::new(),
            debugger_comps: vec![
                DebuggerComponentUi {
                    comp: Box::new(CpuDebuggerComponent::new()),
                },
                DebuggerComponentUi {
                    comp: Box::new(DisassemblerComponent::new(10, 3)),
                },
            ],
            last_clock: Instant::now(),
            refresh_rate: Duration::from_secs_f64(0.01674),
        }
    }

    pub fn update(&mut self, ui: &mut egui::Ui) {
        self.draw_frame();
        self.ui(ui);
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let side_panels_width = ui.available_width() / 4.0;
        let gameboy_snapshot = self.gameboy.get_snapshot();

        egui::Panel::left("Execution State")
            .resizable(false)
            .exact_size(side_panels_width)
            .show(ui, |ui| {
                egui::Frame::new()
                    .fill(PANEL_BG)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for comp in &mut self.debugger_comps {
                                comp.ui(ui, &gameboy_snapshot);
                            }
                        });
                    });
            });

        egui::Panel::right("Right Panel")
            .resizable(false)
            .exact_size(side_panels_width)
            .show(ui, |_| {});

        egui::CentralPanel::default().show(ui, |ui| {
            self.gb_renderer.ui(ui, self.gameboy.get_framebuffer());
        });

        ui.request_repaint_after(self.refresh_rate);
    }

    fn draw_frame(&mut self) {
        if Instant::now() - self.last_clock >= self.refresh_rate {
            self.gameboy.next_frame();
            self.last_clock = Instant::now();
        }
    }
}
