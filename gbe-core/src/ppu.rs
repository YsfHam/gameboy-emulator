use crate::{bus::Bus, constants::{OAM_END, OAM_START}, interrupts::Interrupt, lcd::LCDCFlags, ppu::{fetcher::{FetcherData, PixelsFetcher}, pixels::PixelsPrinter}};

mod fetcher;
mod pixels;

#[derive(Debug, Clone, Copy)]
pub struct OAMEntry {
    y_pos: u8,
    x_pos: u8,
    tile_index: u8,
    flags: u8,
}

struct OAMEntriesFetcher {
    current_addr: u16,
}

impl OAMEntriesFetcher {
    fn new(start_addr: u16) -> Self {
        Self {
            current_addr: start_addr
        }
    }

    fn fetch_entry(&mut self, bus: &Bus) -> OAMEntry {
        let addr = self.current_addr;
        self.current_addr += 4;
        OAMEntry {
            y_pos: bus.read_u8(addr),
            x_pos: bus.read_u8(addr + 1),
            tile_index: bus.read_u8(addr + 2),
            flags: bus.read_u8(addr + 3),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum PpuMode {
    FrameStart,
    Mode2Start,
    Mode2,
    Mode3Start,
    Mode3,
    Mode0,
    Mode0End,
    Mode1,
    Mode1End,
    FrameEnd,
}

pub struct Ppu {
    mode: PpuMode,
    dots: u16,
    mode0_dots: u16,
    frame_completed: bool,

    oam_entries_fetcher: OAMEntriesFetcher,
    oam_entries: Vec<OAMEntry>,

    fetcher: PixelsFetcher,
    pixel_display: PixelsPrinter,

    window_y: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            mode: PpuMode::FrameStart,
            dots: 0,
            mode0_dots: 0,
            frame_completed: false,
            oam_entries_fetcher: OAMEntriesFetcher::new(OAM_START),
            oam_entries: vec![],
            fetcher: PixelsFetcher::new(),
            pixel_display: PixelsPrinter::new(),
            window_y: false,
        }
    }

    pub fn clock(&mut self, bus: &mut Bus) {
        if !bus.lcd.get_lcdc_flag(LCDCFlags::LCDPpuEnable) {
            bus.lcd.reset_ly();
            self.mode = PpuMode::FrameStart;
            self.dots = 0;
            self.pixel_display.reset();
            bus.lcd.reset_stat();
            return;
        }
        for _ in 0..4 {
            if self.dot_clock(bus) {
                break;
            }
        }
    }

    pub fn frame_completed(&self) -> bool {
        self.frame_completed
    }

    fn dot_clock(&mut self, bus: &mut Bus) -> bool {
        while self.dots == 0 {
            self.executing_mode(bus);
            if self.frame_completed {
                return true;
            }
        }
        self.handle_stat_reg(bus);
        self.dots = self.dots.saturating_sub(1);

        false
    }

    fn executing_mode(&mut self, bus: &mut Bus) {
        match self.mode {
            PpuMode::FrameStart => self.frame_start(),
            PpuMode::Mode2Start => self.mode2_start(bus),
            PpuMode::Mode2 => self.mode2(bus),
            PpuMode::Mode3Start => self.mode3_start(bus),
            PpuMode::Mode3 => self.mode3(bus),
            PpuMode::Mode0 => self.mode0(),
            PpuMode::Mode0End => self.mode0_end(bus),
            PpuMode::Mode1 => self.mode1(bus),
            PpuMode::Mode1End => self.mode1_end(bus),
            PpuMode::FrameEnd => self.frame_end(bus),
        }
    }

    fn stat_mode(&self) -> u8 {
        match self.mode {
            PpuMode::Mode0 | PpuMode::Mode0End => 0,
            PpuMode::Mode1 | PpuMode::Mode1End | PpuMode::FrameEnd => 1,
            PpuMode::FrameStart | PpuMode::Mode2Start | PpuMode::Mode2 => 2,
            PpuMode::Mode3Start | PpuMode::Mode3 => 3,
        }
    }

    fn handle_stat_reg(&self, bus: &mut Bus) {
        if bus.lcd.update_stat(self.stat_mode()) {
            bus.interrupts_flags.request_interrupt(Interrupt::LCD);
        }
    }
}

impl Ppu {
    fn fetch_oam_entry(&mut self, bus: &Bus) -> OAMEntry {
        let entry = self.oam_entries_fetcher.fetch_entry(bus);
        if self.oam_entries_fetcher.current_addr > OAM_END {
            self.oam_entries_fetcher.current_addr = OAM_START;
            self.mode = PpuMode::Mode3Start;
        }
        entry
    }
}

impl Ppu {

    fn frame_start(&mut self) {
        self.frame_completed = false;
        self.mode = PpuMode::Mode2Start;
        self.fetcher.new_frame();
        self.window_y = false;
    }

    fn mode2_start(&mut self, bus: &Bus) {
        self.oam_entries.clear();
        self.mode = PpuMode::Mode2;
        self.window_y = self.window_y || bus.window.wy == bus.lcd.ly();
    }

    fn mode2(&mut self, bus: &mut Bus) {

        let obj_size = bus.lcd.get_obj_height();
        let ly = bus.lcd.ly();
        let entry = self.fetch_oam_entry(bus);
        let bounds = entry.y_pos as u16..(entry.y_pos as u16 + obj_size as u16);
        let ly_offseted = ly as u16 + 16;
        if self.oam_entries.len() < 10 && bounds.contains(&ly_offseted) {
            self.oam_entries.push(entry);
        }
        self.dots = 2;
    }

    fn mode3_start(&mut self, bus: &mut Bus) {

        self.oam_entries.sort_by_key(|e| e.x_pos);
        self.pixel_display.reset();
        self.pixel_display.set_fine_scroll_discard(bus.lcd.scx);
        self.fetcher.new_scanline();
        self.mode = PpuMode::Mode3;
        self.dots = 12;
        self.mode0_dots = 12;
    }

    fn mode3(&mut self, bus: &mut Bus) {
        self.fetcher.clock(bus, &mut self.pixel_display, &FetcherData {
            window_y: self.window_y,
            oam_entries: &self.oam_entries
        });
        let switch_mode0 = if self.fetcher.is_fetching_sprite() { false }
        else { self.pixel_display.display_pixel(&mut bus.lcd) };
        
        self.dots = 1;
        self.mode0_dots += 1;


        if switch_mode0 {
            self.mode0_dots = 376 - self.mode0_dots;
            self.mode = PpuMode::Mode0;
        }
    }

    fn mode0(&mut self) {
        self.mode = PpuMode::Mode0End;
        self.dots = self.mode0_dots;
    }

    fn mode0_end(&mut self, bus: &mut Bus) {
        self.fetcher.end_scanline();
        bus.lcd.increment_ly();
        if bus.lcd.ly() == 144 {
            self.mode = PpuMode::Mode1;
        }
        else {
            self.mode = PpuMode::Mode2Start;
        }
    }

    fn mode1(&mut self, bus: &mut Bus) {
        self.dots = 456;
        self.mode = PpuMode::Mode1End;
        bus.interrupts_flags.request_interrupt(Interrupt::VBlank);
    }

    fn mode1_end(&mut self, bus: &mut Bus) {
        if bus.lcd.ly() == 153 {
            self.mode = PpuMode::FrameEnd
        }
        else {
            bus.lcd.increment_ly();
            self.mode = PpuMode::Mode1;
        }
    }

    fn frame_end(&mut self, bus: &mut Bus) {
        self.frame_completed = true;
        self.mode = PpuMode::FrameStart;
        bus.lcd.reset_ly();
    } 

}