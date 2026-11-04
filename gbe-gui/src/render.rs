
use eframe::egui::{self, Color32, ColorImage};
use gbe_core::{SCREEN_HEIGHT, SCREEN_WIDTH};

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

pub struct GameBoyRenderer {
    texture: Option<egui::TextureHandle>,
    palette: Palette,
}

impl GameBoyRenderer {
    pub fn new() -> Self {
        Self {
            texture: None,
            palette: Palette::default(),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, framebuffer: &[u8]) {
        self.present(ui, framebuffer);
    }
    
    fn present(&mut self, ui: &mut egui::Ui, framebuffer: &[u8]) {
        let texture = self.upload_to_texture(ui.ctx(), framebuffer);

        let base_size = texture.size_vec2();
        ui.centered_and_justified(|ui| {
            let size = ui.available_size();
            let ratio_x = size.x / SCREEN_WIDTH as f32;
            let ratio_y = size.y / SCREEN_HEIGHT as f32;
            
            let min_ratio = ratio_x.min(ratio_y);
            let size = base_size * min_ratio;
            let sized_texture = egui::load::SizedTexture::new(texture, size);
            ui.add(egui::Image::new(sized_texture).fit_to_exact_size(size));
        });
    }

    fn upload_to_texture(&mut self, ctx: &egui::Context, framebuffer: &[u8]) -> &mut egui::TextureHandle {
        let mut pixels = vec![Color32::BLACK; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize];

        for (i, fb) in framebuffer.iter().enumerate() {
            let color = self.palette.colors[(*fb % 4) as usize];
            pixels[i] = color;
        }
        let texture = self.get_texture(ctx);
        texture.set(
            ColorImage::new(
                [SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize],
                pixels
            ),
            egui::TextureOptions::NEAREST
        );

        texture
    }

    fn get_texture(&mut self, ctx: &egui::Context) -> &mut egui::TextureHandle {
        self.texture.get_or_insert_with(|| {
            ctx.load_texture(
                "GameboyImage", 
                ColorImage::filled([1, 1], Color32::BLACK), 
                egui::TextureOptions::NEAREST
            )
        })
    }
}