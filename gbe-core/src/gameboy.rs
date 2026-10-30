use crate::{bus::Bus, cartridge::Cartridge, cpu::Cpu};



pub struct Gameboy {
    cpu: Cpu,
    bus: Bus,
}

impl Gameboy {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cpu: Cpu::with_entry_point(0x100),
            bus: Bus::new(cartridge),
        }
    }

    pub fn clock(&mut self) {
        self.cpu.clock(&mut self.bus);
        self.bus.clock();
    }
}