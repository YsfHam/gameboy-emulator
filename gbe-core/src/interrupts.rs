#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Interrupt {
    VBlank = 0,
    LCD,
    Timer,
    Serial,
    Joypad
}

impl Interrupt {
    pub fn handler_addr(&self) -> u16 {
        match self {
            Interrupt::VBlank => 0x40,
            Interrupt::LCD => 0x48,
            Interrupt::Timer => 0x50,
            Interrupt::Serial => 0x58,
            Interrupt::Joypad => 0x60,
        }
    }
}

pub struct InterruptFlags {
    pub interrupt_enabled: u8,
    pub interrupt_requested: u8,
}

impl InterruptFlags {
    pub fn new() -> Self {
        Self {
            interrupt_enabled: 0,
            interrupt_requested: 0,
        }
    }

    pub fn any_interrupt_pending(&self) -> bool {
        self.interrupt_enabled & self.interrupt_requested > 0
    }

    pub fn is_interrupt_pending(&self, interrupt: Interrupt) -> bool {
        let pending = self.interrupt_enabled & self.interrupt_requested;
        pending & (1 << interrupt as u8) > 0
    }

}