
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, ColorImage};
use gbe_core::{Gameboy, SCREEN_HEIGHT, SCREEN_WIDTH};

use crate::controller::KeyboardBindings;

pub struct Palette {
    colors: [Color32; 5],
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            colors: [
                Color32::from_rgb(0x9b, 0xbc, 0x0f),
                Color32::from_rgb(0x8b, 0xac, 0x0f),
                Color32::from_rgb(0x30, 0x62, 0x30),
                Color32::from_rgb(0x0f, 0x38, 0x0f),
                Color32::from_rgb(0xff, 0x00, 0xff)
            ],
        }
    }
}

struct GameboyRenderer {
    texture: egui::TextureHandle,
    palette: Palette,
}

impl GameboyRenderer {
    pub fn new(ctx: &egui::Context) -> Self {
        Self {
            texture: ctx.load_texture(
                "GameboyImage", 
                ColorImage::filled([SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize], Color32::BLACK), 
                egui::TextureOptions::NEAREST
            ),
            palette: Palette::default(),
        }
    }

    fn ui(&self, ui: &mut egui::Ui) {
        let base_size = self.texture.size_vec2();
        ui.centered_and_justified(|ui| {
            let size = ui.available_size();
            let ratio_x = size.x / SCREEN_WIDTH as f32;
            let ratio_y = size.y / SCREEN_HEIGHT as f32;
            
            let min_ratio = ratio_x.min(ratio_y);
            let size = base_size * min_ratio;
            let sized_texture = egui::load::SizedTexture::new(&self.texture, size);
            ui.add(egui::Image::new(sized_texture).fit_to_exact_size(size));
        });
    }

    fn upload_to_texture(&mut self, framebuffer: &[u8]) {
        let mut pixels = vec![Color32::BLACK; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize];

        for (i, fb) in framebuffer.iter().enumerate() {
            let color = self.palette.colors[(*fb % 4) as usize];
            pixels[i] = color;
        }
        self.texture.set(
            ColorImage::new(
                [SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize],
                pixels
            ),
            egui::TextureOptions::NEAREST
        );
    }
}


const GAMEBOY_FRAME_TIME: f64 = 0.01674;

pub struct GameboyRunner {
    last_clock: Instant,
    frame_duration: Duration,
    renderer: GameboyRenderer,
    keyboard_bindings: KeyboardBindings,
}

impl GameboyRunner {
    pub fn new(ctx: &egui::Context) -> Self {
        Self {
            last_clock: Instant::now(),
            frame_duration: Duration::from_secs_f64(GAMEBOY_FRAME_TIME),
            renderer: GameboyRenderer::new(ctx),
            keyboard_bindings: KeyboardBindings::default()
        }
    }

    pub fn frame_duration(&self) -> Duration {
        self.frame_duration
    }

    pub fn update(&mut self, gameboy: &mut Gameboy, input: &egui::InputState) {

        if let Some((joyp, state)) = self.keyboard_bindings.poll(input) {
            match state {
                crate::controller::KeyState::Pressed => gameboy.presse_button(joyp),
                crate::controller::KeyState::Released => gameboy.release_button(joyp),
            }
        }

        let elapsed = self.last_clock.elapsed();
        if elapsed >= self.frame_duration {
            self.last_clock = Instant::now();
            gameboy.next_frame();
        }

        self.renderer.upload_to_texture(gameboy.get_framebuffer());
    }

    pub fn ui(&self, ui: &mut egui::Ui) {
        self.renderer.ui(ui);
    }

    pub fn ui_with_framebuffer(&mut self, ui: &mut egui::Ui, framebuffer: &[u8]) {
        self.renderer.upload_to_texture(framebuffer);
        self.ui(ui);
    }
}