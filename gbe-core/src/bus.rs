use crate::{cartridge::Cartridge, constants::{ECHO_RAM_END, ECHO_RAM_START, EXTERNAL_RAM_END, EXTERNAL_RAM_START, HRAM_MEM_END, HRAM_MEM_START, IE_REGISTER, IO_INTERRUPTS, VRAM_MEM_END, VRAM_MEM_START, WRAM_BANK_MEM_END, WRAM_BANK_MEM_START, WRAM_MEM_END, WRAM_MEM_START}, interrupts::InterruptFlags, memory::{Ram, Ram4Kib, Ram8Kib}};

pub struct Bus {

    cartridge: Cartridge,

    vram: Ram8Kib,
    wram1: Ram4Kib,
    wram2: Ram4Kib,
    hram: Ram<127>,

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
            self.wram1.read_u8(addr - ECHO_RAM_START)
        }
        else if (HRAM_MEM_START..=HRAM_MEM_END).contains(&addr) {
            self.hram.read_u8(addr - HRAM_MEM_START)
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
        println!("write to address {:x}", addr);
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
            self.wram1.write_u8(addr - ECHO_RAM_START, value);
        }
        else if (HRAM_MEM_START..=HRAM_MEM_END).contains(&addr) {
            self.hram.write_u8(addr - HRAM_MEM_START, value);
        }
        else if addr == IO_INTERRUPTS {
            self.interrupts_flags.interrupt_requested = value;
        }
        else if addr == IE_REGISTER {
            self.interrupts_flags.interrupt_enabled = value;
        }
    }
}
