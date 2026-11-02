
use crate::{bus::Bus, lcd::LCDCFlags, ppu::{OAMEntry, pixels::{PixelsPrinter, SpritePalette, SpritePixel}}};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum FetchingMode {
    BGW,
    Sprite
}

pub struct PixelsFetcher {
    bgw: BGWFetcher,
    sprite_fetcher: SpriteFetcher,
    mode: FetchingMode,
    drawn: [bool; 10],
}

impl PixelsFetcher {

    pub fn new() -> Self {
        Self {
            bgw: BGWFetcher::new(),
            sprite_fetcher: SpriteFetcher::new(),
            mode: FetchingMode::BGW,
            drawn: [false; 10],
        }
    }

    pub fn clock(&mut self, bus: &Bus, pixels: &mut PixelsPrinter, data: &FetcherData) {
        if self.mode == FetchingMode::BGW {
            if let Some(i) = self.find_sprite(bus, pixels, data.oam_entries) {
                self.drawn[i] = true;
                self.sprite_fetcher.start(data.oam_entries[i], pixels.output_x);
                self.mode = FetchingMode::Sprite;
            }
        }
        match self.mode {
            FetchingMode::BGW => self.bgw.clock(bus, pixels, data),
            FetchingMode::Sprite => {
                if self.sprite_fetcher.clock(bus, pixels) {
                    self.mode = FetchingMode::BGW;
                }
            }
        }
    }

    pub fn is_fetching_sprite(&self) -> bool {
        self.mode == FetchingMode::Sprite
    }

    pub fn new_frame(&mut self) {
        self.bgw.new_frame();
    } 

    pub fn new_scanline(&mut self) {
        self.bgw.new_scanline();
        self.drawn = [false; 10];
        self.mode = FetchingMode::BGW;
    }    

    pub fn end_scanline(&mut self) {
        self.bgw.end_scanline();
    }

    fn find_sprite(&self, bus: &Bus, pixels: &PixelsPrinter, entries: &[OAMEntry]) -> Option<usize> {
        if entries.is_empty() || !bus.lcd.get_lcdc_flag(LCDCFlags::ObjEnable) {
            return None;
        }
        let limit = pixels.output_x as u16 + 8;
        entries.iter().enumerate()
            .find(|(i, e)| !self.drawn[*i] && e.x_pos as u16 <= limit)
            .map(|(i, _)| i)
    }
}

pub struct FetcherData<'a> {
    pub window_y: bool,
    pub oam_entries: &'a [OAMEntry]
}

#[derive(Debug, Clone, Copy)]
enum FetcherStage {
    FetchTileNumber,
    FetchLowByte,
    FetchHighByte,
    PushFifo
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BGWFetchSource {
    Window,
    Background,
}

struct BGWFetcher {
    fetcher_x: u8,
    tile_number: u8,
    bg_y: u8,
    row_addr: u16,
    low_byte: u8,
    high_byte: u8,
    dots: u8,
    stage: FetcherStage,
    window_line_counter: u8,
    fetch_source: BGWFetchSource,
}

impl BGWFetcher {

    fn new() -> Self {
        Self {
            fetcher_x: 0,
            tile_number: 0,
            bg_y: 0,
            row_addr: 0,
            low_byte: 0,
            high_byte: 0,
            dots: 0,
            stage: FetcherStage::FetchTileNumber,
            window_line_counter: 0,
            fetch_source: BGWFetchSource::Background,
        }
    }

    fn new_scanline(&mut self) {
        self.fetcher_x = 0;
        self.stage = FetcherStage::FetchTileNumber;
        self.dots = 0;
        self.fetch_source = BGWFetchSource::Background;
    }

    fn end_scanline(&mut self) {
        if self.fetch_source == BGWFetchSource::Window {
            self.window_line_counter += 1;
        }
    }

    fn reset_bg_state(&mut self, pixels: &mut PixelsPrinter) {
        self.fetcher_x = 0;
        self.stage = FetcherStage::FetchTileNumber;
        self.dots = 0;
        pixels.clear_bg_fifo();
    }

    fn new_frame(&mut self) {
        self.window_line_counter = 0;
    }

    fn clock(&mut self, bus: &Bus, pixels: &mut PixelsPrinter, data: &FetcherData) {
        self.check_window_switch(bus, pixels, data);   // every dot
        if self.dots == 0 {
            self.execute_stage(bus, pixels);
        }
        self.dots = self.dots.saturating_sub(1);
    }

    fn check_window_switch(&mut self, bus: &Bus, pixels: &mut PixelsPrinter, data: &FetcherData) {
        let window_active = bus.lcd.get_lcdc_flag(LCDCFlags::WindowEnable)
            && data.window_y
            && pixels.output_x as u16 + 7 >= bus.window.wx as u16;
        let target = if window_active { BGWFetchSource::Window } else { BGWFetchSource::Background };

        if target != self.fetch_source {
            self.fetch_source = target;
            self.reset_bg_state(pixels);
            if target == BGWFetchSource::Window {
                pixels.cancel_fine_scroll();
            }
        }
    }

    fn execute_stage(&mut self, bus: &Bus, pixels: &mut PixelsPrinter) {
        match self.stage {
            FetcherStage::FetchTileNumber => self.fetch_bg_tile_number(bus),
            FetcherStage::FetchLowByte => self.fetch_low_byte(bus),
            FetcherStage::FetchHighByte => self.fetch_high_byte(bus),
            FetcherStage::PushFifo => self.push_fifo(bus, pixels),
        }
    }

    fn fetch_bg_tile_number(&mut self, bus: &Bus) {
        let lcd = &bus.lcd;
        let (bg_y, tile_x, base) = match self.fetch_source {
            BGWFetchSource::Background => (
                lcd.scy.wrapping_add(lcd.ly()),
                ((lcd.scx / 8) + self.fetcher_x) & 31,
                lcd.get_bg_tile_map_base(),
            ),
            BGWFetchSource::Window => (
                self.window_line_counter,
                self.fetcher_x & 31,
                lcd.get_window_tile_map_base(),
            ),
        };
        self.bg_y = bg_y;
        let map_addr = base + (bg_y / 8) as u16 * 32 + tile_x as u16;
        self.tile_number = bus.read_u8(map_addr);
        self.stage = FetcherStage::FetchLowByte;
        self.dots = 2;
    }

    fn fetch_low_byte(&mut self, bus: &Bus) {
        let row = self.bg_y % 8;
        let lcd = &bus.lcd;
        let tile_addr = if lcd.get_lcdc_flag(LCDCFlags::BGWindowTiles) {
            0x8000 + self.tile_number as u16 * 16
        }
        else {
            let tile_number = self.tile_number as i8 as i16;
            (0x9000 as u16).wrapping_add_signed(tile_number * 16)
        };

        self.row_addr = tile_addr + row as u16 * 2;
        self.low_byte = bus.read_u8(self.row_addr);

        self.stage = FetcherStage::FetchHighByte;
        self.dots = 2;
    }

    fn fetch_high_byte(&mut self, bus: &Bus) {
        self.high_byte = bus.read_u8(self.row_addr + 1);
        self.stage = FetcherStage::PushFifo;
        self.dots = 2;
    }

    fn push_fifo(&mut self, bus: &Bus, pixels: &mut PixelsPrinter) {
        if !pixels.is_bg_fifo_empty() {
            self.dots = 1;
            return;
        }
        
        for i in 0..8 {
            let color = if bus.lcd.get_lcdc_flag(LCDCFlags::BGWindowEnable) {
                let bit = 7 - i;
                let color = (((self.high_byte >> bit) & 1) << 1) | ((self.low_byte >> bit) & 1);
                color
            }
            else {
                0
            };
            pixels.push_bg_color(color);
        }
        self.fetcher_x = self.fetcher_x.wrapping_add(1);
        self.stage = FetcherStage::FetchTileNumber;
        self.dots = 2;
    }
}


struct SpriteFetcher {
    sprite: OAMEntry,
    dots_left: u8,
    skip: u8,
}

impl SpriteFetcher {
    fn new() -> Self {
        Self {
            sprite: OAMEntry { y_pos: 0, x_pos: 0, tile_index: 0, flags: 0 },
            dots_left: 0,
            skip: 0,
        }
    }

    fn start(&mut self, sprite: OAMEntry, output_x: u8) {
        self.sprite = sprite;
        self.skip = (output_x as u16 + 8 - sprite.x_pos as u16).min(8) as u8;
        self.dots_left = 6;
    }

    fn clock(&mut self, bus: &Bus, pixels: &mut PixelsPrinter) -> bool {
        self.dots_left -= 1;
        if self.dots_left > 0 {
            return false;
        }
        let s = self.sprite;
        let height = bus.lcd.get_obj_height() as u16;
        let mut row = bus.lcd.ly() as u16 + 16 - s.y_pos as u16;
        if (s.flags >> 6) & 1 == 1 { row = height - 1 - row; }
        let tile = if height == 16 { s.tile_index & 0xFE } else { s.tile_index };
        let addr = 0x8000 + tile as u16 * 16 + row * 2;
        let low = bus.read_u8(addr);
        let high = bus.read_u8(addr + 1);

        let palette = if (s.flags >> 4) & 1 == 0 { SpritePalette::Obj0 } else { SpritePalette::Obj1 };
        let priority = (s.flags >> 7) & 1 == 1;
        let x_flip = (s.flags >> 5) & 1 == 1;

        let row_pixels: [SpritePixel; 8] = std::array::from_fn(|i| {
            let bit = if x_flip { i } else { 7 - i };
            SpritePixel { palette, priority, color: (((high >> bit) & 1) << 1) | ((low >> bit) & 1) }
        });
        pixels.merge_sprite_row(&row_pixels[self.skip as usize..]);
        true
    }
}