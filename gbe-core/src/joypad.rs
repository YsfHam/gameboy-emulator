pub struct Joypad {
    keys: u8,
    d_pad: bool,
    buttons: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            keys: 0xFF,
            d_pad: true,
            buttons: true,
        }
    }

    pub(crate) fn set(&mut self, value: u8) {
        self.d_pad = value & 0x10 == 0;
        self.buttons = value & 0x20 == 0;
    }

    pub fn get(&self) -> u8 {
        let mut result = 0xFF;
        if self.d_pad {
            result &= (0b1111 & (self.keys >> 4)) | 0b1110_0000
        }
        if self.buttons {
            result &= (0b1111 & (self.keys & 0xF)) | 0b1101_0000
        }
        result
    }

    pub(crate) fn presse_button(&mut self, btn: JoypButton) {
        self.keys = self.keys & !(1 << btn as u8);
    }

    pub(crate) fn release_button(&mut self, btn: JoypButton) {
        self.keys = self.keys | (1 << btn as u8);
    }

    pub fn is_button_pressed(&self, btn: JoypButton) -> bool {
        self.keys & (1 << btn as u8) == 0
    }
    
    pub(crate) fn any_selected_pressed(&self) -> bool {
        self.get() & 0x0F != 0x0F
    }

    pub fn select_d_pad(&self) -> bool {
        self.d_pad
    }

    pub fn select_button(&self) -> bool {
        self.buttons
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum JoypButton {
    A = 0,
    B,
    Select,
    Start,

    Right,
    Left,
    Up,
    Down
}