use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LCDCFlags {
    BGWindowEnable = 0,
    ObjEnable,
    ObjSize,
    BGTileMap,
    BGWindowTiles,
    WindowEnable,
    WindowTileMap,
    LCDPpuEnable
}

pub struct LCD {
    pub control: u8,
    lcd_y: u8,
    lcd_y_cp: u8,
    pub stats: u8,
    pub scy: u8,
    pub scx: u8,
    pub bgp: u8,
    pub obp0: u8,
    pub obp1: u8,
    pub framebuffer: [u8; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize],
    stats_line: bool,
}

impl LCD {
    pub fn new() -> Self {
        Self {
            control: 0x91,
            lcd_y: 0,
            lcd_y_cp: 0,
            stats: 0,
            scy: 0,
            scx: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            framebuffer: [0; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize],
            stats_line: false,
        }
    }

    pub fn get_lcdc_flag(&self, flag: LCDCFlags) -> bool {
        self.control & (1 << (flag as u8)) > 0
    }

    pub fn get_obj_height(&self) -> u8 {
        if self.get_lcdc_flag(LCDCFlags::ObjSize) {
            16
        }
        else {
            8
        }
    }

    pub fn get_bg_tile_map_base(&self) -> u16 {
        if self.get_lcdc_flag(LCDCFlags::BGTileMap) {
            0x9C00
        }
        else {
            0x9800
        }
    }

    pub fn get_window_tile_map_base(&self) -> u16 {
        if self.get_lcdc_flag(LCDCFlags::WindowTileMap) {
            0x9C00
        }
        else {
            0x9800
        }
    }

    pub fn ly(&self) -> u8 {
        self.lcd_y
    }

    pub fn increment_ly(&mut self) {
        self.lcd_y = self.lcd_y + 1;
    } 

    pub fn reset_ly(&mut self) {
        self.lcd_y = 0;
    }

    pub fn lyc(&self) -> u8 {
        self.lcd_y_cp
    }

    pub fn set_lyc(&mut self, value: u8) {
        self.lcd_y_cp = value;
    }

    pub fn read_stat(&self) -> u8 {
        self.stats | 0x80 
    }

    pub fn write_stat(&mut self, value: u8) {
        // only bits 3-6 are writable
        self.stats = (self.stats & !0x78) | (value & 0x78);
    }

    pub fn update_stat(&mut self, mode: u8) -> bool {
        self.stats = (self.stats & !0b11) | mode;
        if self.lcd_y == self.lcd_y_cp { self.stats |= 0x04 } else { self.stats &= !0x04 }

        let s = self.stats;
        let line = (s & 0x40 != 0 && s & 0x04 != 0)   // LYC
            || (mode == 0 && s & 0x08 != 0)           // HBlank
            || (mode == 1 && s & 0x10 != 0)           // VBlank
            || (mode == 2 && s & 0x20 != 0);          // OAM
        let rising = line && !self.stats_line;
        self.stats_line = line;
        rising
    }

    pub fn reset_stat(&mut self) {          // call when the LCD is turned off
        self.stats &= !0b111;
        self.stats_line = false;
        self.lcd_y = 0;
    }
}