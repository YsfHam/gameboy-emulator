use std::{cell::RefCell, rc::Rc};

use crate::{cartridge::Cartridge, constants::{BGP, DIV_REGISTER, ECHO_RAM_END, ECHO_RAM_START, EXTERNAL_RAM_END, EXTERNAL_RAM_START, HRAM_MEM_END, HRAM_MEM_START, IE_REGISTER, IO_INTERRUPTS, JOYP, LCD_CONTROL, LCD_STATS, LCD_Y, LCD_Y_COMPARE, OAM_DMA_TRANSFER, OAM_END, OAM_START, OBP0, OBP1, SCROLL_X, SCROLL_Y, SERIAL_SB, TAC, TIMA, TMA, VRAM_MEM_END, VRAM_MEM_START, WINDOW_X, WINDOW_Y, WRAM_BANK_MEM_END, WRAM_BANK_MEM_START, WRAM_MEM_END, WRAM_MEM_START}, debugging::events::{Event, EventBuffer, EventKind}, interrupts::{Interrupt, InterruptFlags}, joypad::Joypad, lcd::LCD, memory::{Ram, Ram4Kib, Ram8Kib}, serial::SerialDevice, timers::Timers, window::Window};

pub struct Bus {

    cartridge: Cartridge,

    vram: Ram8Kib,
    wram1: Ram4Kib,
    wram2: Ram4Kib,
    oam: Ram<160>,
    hram: Ram<127>,

    oam_dma_source_page: u8,
    oam_dma_active: bool,
    oam_dma_byte_index: u8,
    pub div_register: u16,
    pub timers: Timers,

    pub lcd: LCD,

    serial: Option<Rc<RefCell<dyn SerialDevice>>>,

    pub window: Window,

    pub interrupts_flags: InterruptFlags,

    pub vram_locked: bool,
    pub oam_locked: bool,
    pub joypad: Joypad
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cartridge,
            vram: Ram8Kib::new(),
            wram1: Ram4Kib::new(),
            wram2: Ram4Kib::new(),
            oam: Ram::new(),
            hram: Ram::new(),
            div_register: 0,
            lcd: LCD::new(),
            serial: None,
            window: Window::new(),
            timers: Timers::new(),
            interrupts_flags: InterruptFlags::new(),
            oam_dma_source_page: 0,
            oam_dma_active: false,
            oam_dma_byte_index: 0,

            vram_locked: false,
            oam_locked: false,

            joypad: Joypad::new()
        }
    }

    /// Read without VRAM/OAM lock checks; for PPU-internal hardware access.
    pub fn ppu_read_u8(&self, addr: u16) -> u8 {
        if (VRAM_MEM_START..=VRAM_MEM_END).contains(&addr) {
            self.vram.read_u8(addr - VRAM_MEM_START)
        }
        else if (OAM_START..=OAM_END).contains(&addr) {
            if self.oam_dma_active {
                // While an OAM DMA is in progress, the PPU cannot read OAM properly.
                return 0xFF;
            }
            self.oam.read_u8(addr - OAM_START)
        }
        else {
            self.read_u8(addr)
        }
    }

    fn oam_locked(&self) -> bool {
        self.oam_locked || self.oam_dma_active
    }

    /// True while the OAM DMA unit is actually reading/writing a byte, i.e. after
    /// the start-delay M-cycle that follows the $FF46 write.
    fn oam_dma_transferring(&self) -> bool {
        self.oam_dma_active && self.oam_dma_byte_index < 160
    }

    /// True when a CPU access to `addr` collides with an ongoing OAM DMA transfer.
    ///
    /// On DMG the DMA unit shares the CPU bus with ROM, cartridge RAM, WRAM, echo
    /// RAM and OAM: CPU accesses to that shared bus return 0xFF / drop writes while
    /// the transfer runs. VRAM sits on its own bus, so it only conflicts when the
    /// transfer itself is reading from VRAM. HRAM ($FF80-$FFFE) and the IO registers
    /// ($FF00+) are on separate buses and stay accessible (the DMA routine itself
    /// and interrupts must be able to touch $FF46, IF, etc.).
    fn oam_dma_conflict(&self, addr: u16) -> bool {
        if !self.oam_dma_transferring() {
            return false;
        }
        if addr >= 0xFF00 {
            return false;
        }
        let addr_vram = (VRAM_MEM_START..=VRAM_MEM_END).contains(&addr);
        let src_vram = (VRAM_MEM_START..=VRAM_MEM_END)
            .contains(&((self.oam_dma_source_page as u16) << 8));
        addr_vram == src_vram
    }

    /// Raw peek for debug tooling and tests: bypasses all lock and DMA conflict
    /// checks that apply to CPU-visible reads. IO registers have no conflict
    /// semantics, so they route through the normal read path.
    pub fn peek_u8(&self, addr: u16) -> u8 {
        if (VRAM_MEM_START..=VRAM_MEM_END).contains(&addr) {
            self.vram.read_u8(addr - VRAM_MEM_START)
        }
        else if (OAM_START..=OAM_END).contains(&addr) {
            self.oam.read_u8(addr - OAM_START)
        }
        else if addr < VRAM_MEM_START || (EXTERNAL_RAM_START..=EXTERNAL_RAM_END).contains(&addr) {
            self.cartridge.read_u8(addr)
        }
        else if (WRAM_MEM_START..=WRAM_MEM_END).contains(&addr) {
            self.wram1.read_u8(addr - WRAM_MEM_START)
        }
        else if (WRAM_BANK_MEM_START..=WRAM_BANK_MEM_END).contains(&addr) {
            self.wram2.read_u8(addr - WRAM_BANK_MEM_START)
        }
        else if (ECHO_RAM_START..=ECHO_RAM_END).contains(&addr) {
            let addr = addr - ECHO_RAM_START;
            if (addr as usize) < self.wram1.len() {
                self.wram1.read_u8(addr)
            }
            else {
                self.wram2.read_u8(addr - self.wram1.len() as u16)
            }
        }
        else if (HRAM_MEM_START..=HRAM_MEM_END).contains(&addr) {
            self.hram.read_u8(addr - HRAM_MEM_START)
        }
        else {
            self.read_u8(addr)
        }
    }

    /// Raw poke for debug tooling and tests: bypasses all lock and DMA conflict
    /// checks that apply to CPU-visible writes.
    pub fn poke_u8(&mut self, addr: u16, value: u8) {
        if (VRAM_MEM_START..=VRAM_MEM_END).contains(&addr) {
            self.vram.write_u8(addr - VRAM_MEM_START, value);
        }
        else if (OAM_START..=OAM_END).contains(&addr) {
            self.oam.write_u8(addr - OAM_START, value);
        }
        else if (WRAM_MEM_START..=WRAM_MEM_END).contains(&addr) {
            self.wram1.write_u8(addr - WRAM_MEM_START, value);
        }
        else if (WRAM_BANK_MEM_START..=WRAM_BANK_MEM_END).contains(&addr) {
            self.wram2.write_u8(addr - WRAM_BANK_MEM_START, value);
        }
        else if (HRAM_MEM_START..=HRAM_MEM_END).contains(&addr) {
            self.hram.write_u8(addr - HRAM_MEM_START, value);
        }
        else if addr == LCD_CONTROL {
            self.lcd.control = value;
        }
        else {
            self.write_u8(addr, value);
        }
    }

    pub fn reset(&mut self) {
        self.div_register = 0;
        self.lcd = LCD::new();
        self.window = Window::new();
        self.timers = Timers::new();
        self.interrupts_flags = InterruptFlags::new();
        self.oam_dma_source_page = 0;
        self.oam_dma_active = false;
        self.oam_dma_byte_index = 0;
        self.vram_locked = false;
        self.oam_locked = false;
        self.joypad = Joypad::new();
    }

    pub fn plug_serial_device<SD: SerialDevice + 'static>(&mut self, serial_device: SD) -> Rc<RefCell<SD>> {
        let serial_device_ref = Rc::new(RefCell::new(serial_device));
        self.serial = Some(serial_device_ref.clone());
        serial_device_ref
    } 

    pub fn remove_serial_device(&mut self) {
        self.serial = None;
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        if self.oam_dma_conflict(addr) {
            // The DMA unit is using this address's bus; the CPU read returns 0xFF.
            return 0xFF;
        }
        if addr < VRAM_MEM_START {
            self.cartridge.read_u8(addr)
        }
        else if (VRAM_MEM_START..=VRAM_MEM_END).contains(&addr) && !self.vram_locked {
            self.vram.read_u8(addr - VRAM_MEM_START)
        }
        else if (EXTERNAL_RAM_START..=EXTERNAL_RAM_END).contains(&addr) {
            self.cartridge.read_u8(addr)
        }
        else if (WRAM_MEM_START..=WRAM_MEM_END).contains(&addr) {
            self.wram1.read_u8(addr - WRAM_MEM_START)
        }
        else if (WRAM_BANK_MEM_START..=WRAM_BANK_MEM_END).contains(&addr) {
            self.wram2.read_u8(addr - WRAM_BANK_MEM_START)
        }
        else if (OAM_START..=OAM_END).contains(&addr) {
            if self.oam_locked() {
                return 0xFF;
            }
            self.oam.read_u8(addr - OAM_START)
        }
        else if (ECHO_RAM_START..=ECHO_RAM_END).contains(&addr) {
            let addr = addr - ECHO_RAM_START;
            if (addr as usize) < self.wram1.len() {
                self.wram1.read_u8(addr)
            }
            else {
                self.wram2.read_u8(addr - self.wram1.len() as u16)
            }
        }
        else if (HRAM_MEM_START..=HRAM_MEM_END).contains(&addr) {
            self.hram.read_u8(addr - HRAM_MEM_START)
        }
        else if addr == DIV_REGISTER {
            (self.div_register >> 8) as u8
        }
        else if addr == TIMA {
            self.timers.tima()
        }
        else if addr == TMA {
            self.timers.tma
        }
        else if addr == TAC {
            self.timers.tac
        }
        else if addr == IO_INTERRUPTS {
            self.interrupts_flags.interrupt_requested | 0xE0
        }
        else if addr == IE_REGISTER {
            self.interrupts_flags.interrupt_enabled
        }
        else if addr == LCD_CONTROL {
            self.lcd.control
        }
        else if addr == LCD_Y {
            self.lcd.ly()
        }
        else if addr == LCD_Y_COMPARE {
            self.lcd.lyc()
        }
        else if addr == LCD_STATS {
            self.lcd.read_stat()
        }
        else if addr == SCROLL_X {
            self.lcd.scx
        }
        else if addr == SCROLL_Y {
            self.lcd.scy
        }
        else if addr == BGP {
            self.lcd.bgp
        }
        else if addr == OBP0 {
            self.lcd.obp0
        }
        else if addr == OBP1 {
            self.lcd.obp1
        }
        else if addr == WINDOW_X {
            self.window.wx
        }
        else if addr == WINDOW_Y {
            self.window.wy
        }
        else if addr == JOYP {
            self.joypad.get()
        }
        else {
            0xFF
        }
    }

    pub fn write_u8(&mut self, addr: u16, value: u8) {
        if self.oam_dma_conflict(addr) {
            // The DMA unit is using this address's bus; the write is dropped.
            return;
        }
        if addr < VRAM_MEM_START {
            self.cartridge.write_u8(addr, value);
        }
        else if (VRAM_MEM_START..=VRAM_MEM_END).contains(&addr) && !self.vram_locked {
            self.vram.write_u8(addr - VRAM_MEM_START, value);
        }
        else if (EXTERNAL_RAM_START..=EXTERNAL_RAM_END).contains(&addr) {
            self.cartridge.write_u8(addr, value)
        }
        else if (WRAM_MEM_START..=WRAM_MEM_END).contains(&addr) {
            self.wram1.write_u8(addr - WRAM_MEM_START, value);
        }
        else if (WRAM_BANK_MEM_START..=WRAM_BANK_MEM_END).contains(&addr) {
            self.wram2.write_u8(addr - WRAM_BANK_MEM_START, value);
        }
        else if (OAM_START..=OAM_END).contains(&addr) {
            if self.oam_locked() {
                return;
            }
            self.oam.write_u8(addr - OAM_START, value)
        }
        else if (ECHO_RAM_START..=ECHO_RAM_END).contains(&addr) {
            let addr = addr - ECHO_RAM_START;
            if (addr as usize) < self.wram1.len() {
                self.wram1.write_u8(addr, value)
            }
            else {
                self.wram2.write_u8(addr - self.wram1.len() as u16, value)
            }
        }
        else if (HRAM_MEM_START..=HRAM_MEM_END).contains(&addr) {
            self.hram.write_u8(addr - HRAM_MEM_START, value);
        }
        else if addr == DIV_REGISTER {
            self.div_register = 0;
        }
        else if addr == TIMA {
            self.timers.set_tima(value);
        }
        else if addr == TMA {
            self.timers.tma = value;
        }
        else if addr == TAC {
            self.timers.tac = value;
        }
        else if addr == IO_INTERRUPTS {
            self.interrupts_flags.interrupt_requested = value;
        }
        else if addr == SERIAL_SB {
            if let Some(serial_device) = &self.serial {
                serial_device.borrow_mut().write(value);
            }
        }
        else if addr == IE_REGISTER {
            self.interrupts_flags.interrupt_enabled = value;
        }
        else if addr == LCD_CONTROL {
            self.lcd.control = value;
        }
        else if addr == LCD_Y_COMPARE {
            self.lcd.set_lyc(value);
        }
        else if addr == LCD_STATS {
            self.lcd.write_stat(value);
        }
        else if addr == SCROLL_X {
            self.lcd.scx = value
        }
        else if addr == SCROLL_Y {
            self.lcd.scy = value;
        }
        else if addr == BGP {
            self.lcd.bgp = value;
        }
        else if addr == OBP0 {
            self.lcd.obp0 = value;
        }
        else if addr == OBP1 {
            self.lcd.obp1 = value;
        }
        else if addr == WINDOW_X {
            self.window.wx = value;
        }
        else if addr == WINDOW_Y {
            self.window.wy = value;
        }
        else if addr == OAM_DMA_TRANSFER {
            self.oam_dma_source_page = value;
            // The transfer begins on the M-cycle after this one, not during this write.
            self.oam_dma_byte_index = 160;
            self.oam_dma_active = true;
        }
        else if addr == JOYP {
            self.joypad.set(value);
            if self.joypad.any_selected_pressed() {
                self.interrupts_flags
                    .request_interrupt(Interrupt::Joypad);
            }
        }
    }

    pub fn reset_div_register(&mut self) {
        self.div_register = 0;
    }

    pub(crate) fn clock(&mut self, events: &mut EventBuffer, cycle: u64) {
        for _ in 0..4 {
            self.timers_tick(events, cycle);
        }
        self.clock_oam_dma();
    }

    /// Copies one byte of the OAM DMA per dot (M-cycle): 160 bytes in 160 M-cycles,
    /// starting on the M-cycle after the $FF46 write. The byte index counts both the
    /// start-delay M-cycle (160) and the remaining bytes to copy.
    fn clock_oam_dma(&mut self) {
        if !self.oam_dma_active {
            return;
        }
        // The write's own M-cycle transfers nothing; the transfer begins next M-cycle.
        if self.oam_dma_byte_index == 160 {
            self.oam_dma_byte_index = 0;
            return;
        }
        let index = self.oam_dma_byte_index as u16;
        let mut src = ((self.oam_dma_source_page as u16) << 8) | index;

        // The DMA source read bypasses the CPU-side locks: pages above $DF are echo
        // RAM on DMG and fold onto WRAM. $FE00-$FFFF cannot be a source.
        if src >= 0xE000 {
            src -= 0x2000;
        }
        let byte = if (VRAM_MEM_START..=VRAM_MEM_END).contains(&src) {
            self.vram.read_u8(src - VRAM_MEM_START)
        }
        else if (WRAM_BANK_MEM_START..=WRAM_BANK_MEM_END).contains(&src) {
            self.wram2.read_u8(src - WRAM_BANK_MEM_START)
        }
        else if (WRAM_MEM_START..=WRAM_MEM_END).contains(&src) {
            self.wram1.read_u8(src - WRAM_MEM_START)
        }
        else {
            // ROM (0x0000-0x7FFF) and cartridge external RAM (0xA000-0xBFFF)
            self.cartridge.read_u8(src)
        };
        self.oam.write_u8(index, byte);
        self.oam_dma_byte_index += 1;
        if self.oam_dma_byte_index == 160 {
            self.oam_dma_active = false;
        }
    }

    fn timers_tick(&mut self, events: &mut EventBuffer, cycle: u64) {
        let request_interrupt = self.timers.increment_tima(self.div_register);
        if request_interrupt {
            self.interrupts_flags.request_interrupt(Interrupt::Timer);
            events.push(Event::new(
                cycle,
                EventKind::TimerReloaded {
                    value: self.timers.tima(),
                },
            ));
        }
        self.div_register = self.div_register.wrapping_add(1);
    }
}
