use crate::{cartridge::Cartridge, constants::{DIV_REGISTER, ECHO_RAM_END, ECHO_RAM_START, EXTERNAL_RAM_END, EXTERNAL_RAM_START, HRAM_MEM_END, HRAM_MEM_START, IE_REGISTER, IO_INTERRUPTS, SERIAL_SB, SERIAL_SC, TAC, TIMA, TMA, VRAM_MEM_END, VRAM_MEM_START, WRAM_BANK_MEM_END, WRAM_BANK_MEM_START, WRAM_MEM_END, WRAM_MEM_START}, interrupts::{Interrupt, InterruptFlags}, memory::{Ram, Ram4Kib, Ram8Kib}, timers::Timers};

pub struct Bus {

    cartridge: Cartridge,

    vram: Ram8Kib,
    wram1: Ram4Kib,
    wram2: Ram4Kib,
    hram: Ram<127>,

    div_register: u16,
    timers: Timers,

    pub interrupts_flags: InterruptFlags,
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cartridge,
            vram: Ram8Kib::new(),
            wram1: Ram4Kib::new(),
            wram2: Ram4Kib::new(),
            hram: Ram::new(),
            div_register: 0,
            timers: Timers::new(),
            interrupts_flags: InterruptFlags::new(),
        }
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
            self.interrupts_flags.interrupt_requested
        }
        else if addr == IE_REGISTER {
            self.interrupts_flags.interrupt_enabled
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
            print!("{}", value as char);
        }
        else if addr == SERIAL_SC {
        }
        else if addr == IE_REGISTER {
            self.interrupts_flags.interrupt_enabled = value;
        }
    }

    pub fn reset_div_register(&mut self) {
        self.div_register = 0;
    }

    pub fn clock(&mut self) {
        for _ in 0..4 {
            self.timers_tick();
        } 
    }

    fn timers_tick(&mut self) {
        let request_interrupt = self.timers.increment_tima(self.div_register);
        if request_interrupt {
            self.interrupts_flags.request_interrupt(Interrupt::Timer);
        }
        self.div_register = self.div_register.wrapping_add(1);
    }
}
