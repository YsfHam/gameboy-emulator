use std::collections::VecDeque;

use crate::{constants::SCREEN_WIDTH, lcd::LCD};

#[derive(Debug, Clone, Copy)]
pub enum SpritePalette {
    Obj0,
    Obj1,
}

#[derive(Debug, Clone, Copy)]
pub struct SpritePixel {
    pub palette: SpritePalette,
    pub priority: bool,
    pub color: u8,
}

impl SpritePixel {
    fn transparent() -> Self {
        Self { palette: SpritePalette::Obj0, priority: false, color: 0 }
    }
}

pub struct PixelsPrinter {
    bg_fifo: VecDeque<u8>,
    sprite_fifo: VecDeque<SpritePixel>,
    pub output_x: u8,
    fine_scroll_discard: u8,
}

impl PixelsPrinter {
    pub fn new() -> Self {
        Self {
            bg_fifo: VecDeque::new(),
            sprite_fifo: VecDeque::new(),
            fine_scroll_discard: 0,
            output_x: 0,
        }
    }

    pub fn reset(&mut self) {
        self.output_x = 0;
        self.bg_fifo.clear();
        self.sprite_fifo.clear();
    }

    pub fn is_bg_fifo_empty(&self) -> bool {
        self.bg_fifo.is_empty()
    }

    pub fn push_bg_color(&mut self, color: u8) {
        self.bg_fifo.push_back(color);
    }

    /// The first sprite fetched wins; later sprites only fill its transparent pixels.
    pub fn merge_sprite_row(&mut self, row: &[SpritePixel]) {
        while self.sprite_fifo.len() < row.len() {
            self.sprite_fifo.push_back(SpritePixel::transparent());
        }
        for (slot, new) in self.sprite_fifo.iter_mut().zip(row) {
            if slot.color == 0 { *slot = *new; }
        }
    }

    pub fn clear_bg_fifo(&mut self) {
        self.bg_fifo.clear();
    }

    pub fn set_fine_scroll_discard(&mut self, scx: u8) {
        self.fine_scroll_discard = scx & 7;
    }

    pub fn cancel_fine_scroll(&mut self) {
        self.fine_scroll_discard = 0;
    }

    pub fn display_pixel(&mut self, lcd: &mut LCD) -> bool {
        let Some(bg) = self.bg_fifo.pop_front() else { return false };
        if self.fine_scroll_discard > 0 {
            self.fine_scroll_discard -= 1;
            return false;
        }
        let (color, palette) = match self.sprite_fifo.pop_front() {
            Some(s) if s.color != 0 && !(s.priority && bg != 0) => {
                let obp = match s.palette { SpritePalette::Obj0 => lcd.obp0, SpritePalette::Obj1 => lcd.obp1 };
                (s.color, obp)
            }
            _ => (bg, lcd.bgp),
        };
        lcd.framebuffer[lcd.ly() as usize * SCREEN_WIDTH as usize + self.output_x as usize] =
            Self::apply_bg_palette(color, palette);
        self.output_x += 1;
        self.output_x as u16 == SCREEN_WIDTH
    }

    fn apply_bg_palette(color: u8, palette: u8) -> u8 {
        palette >> (color << 1) & 3
    }
}