use std::{cell::RefCell, rc::Rc};

use crate::{cartridge::Cartridge, constants::{BGP, DIV_REGISTER, ECHO_RAM_END, ECHO_RAM_START, EXTERNAL_RAM_END, EXTERNAL_RAM_START, HRAM_MEM_END, HRAM_MEM_START, IE_REGISTER, IO_INTERRUPTS, LCD_CONTROL, LCD_STATS, LCD_Y, LCD_Y_COMPARE, OAM_DMA_TRANSFER, OAM_END, OAM_START, OBP0, OBP1, SCROLL_X, SCROLL_Y, SERIAL_SB, TAC, TIMA, TMA, VRAM_MEM_END, VRAM_MEM_START, WINDOW_X, WINDOW_Y, WRAM_BANK_MEM_END, WRAM_BANK_MEM_START, WRAM_MEM_END, WRAM_MEM_START}, debugging::events::{Event, EventBuffer, EventKind}, interrupts::{Interrupt, InterruptFlags}, lcd::LCD, memory::{Ram, Ram4Kib, Ram8Kib}, serial::SerialDevice, timers::Timers, window::Window};

pub struct Bus {

    cartridge: Cartridge,

    vram: Ram8Kib,
    wram1: Ram4Kib,
    wram2: Ram4Kib,
    oam: Ram<160>,
    hram: Ram<127>,

    oam_dma_transfer_cycles: u8,
    div_register: u16,
    timers: Timers,

    pub lcd: LCD,

    serial: Option<Rc<RefCell<dyn SerialDevice>>>,

    pub window: Window,

    pub interrupts_flags: InterruptFlags,
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
            oam_dma_transfer_cycles: 0,
        }
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
        if addr < VRAM_MEM_START {
            self.cartridge.read_u8(addr)
        }
        else if (VRAM_MEM_START..=VRAM_MEM_END).contains(&addr) {
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
        else {
            0xFF
        }
    }

    pub fn write_u8(&mut self, addr: u16, value: u8) {
        if addr < VRAM_MEM_START {
            self.cartridge.write_u8(addr, value);
        }
        else if (VRAM_MEM_START..=VRAM_MEM_END).contains(&addr) {
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
            self.oam.write_u8(addr - OAM_START, value);
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
            self.oam_dma_transfer(value);
            self.oam_dma_transfer_cycles = 160;
        }
    }

    pub fn reset_div_register(&mut self) {
        self.div_register = 0;
    }

    pub(crate) fn clock(&mut self, events: &mut EventBuffer, cycle: u64) {
        for _ in 0..4 {
            self.timers_tick(events, cycle);
        }
        self.oam_dma_transfer_cycles = 
            self.oam_dma_transfer_cycles
            .saturating_sub(4);
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

    fn oam_dma_transfer(&mut self, addr_base: u8) {
        let addr = (addr_base as u16) << 8;
        for i in 0x00..=0x9F {
            let data = self.read_u8(addr | i);
            self.write_u8(OAM_START | i, data);
        }
    }
}
