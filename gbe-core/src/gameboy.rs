use std::{cell::RefCell, rc::Rc};

use crate::{bus::Bus, cartridge::Cartridge, cpu::Cpu, ppu::Ppu, serial::SerialDevice};



pub struct Gameboy {
    cpu: Cpu,
    bus: Bus,
    ppu: Ppu,
}

impl Gameboy {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cpu: Cpu::with_entry_point(0x100),
            bus: Bus::new(cartridge),
            ppu: Ppu::new(),
        }
    }

    pub fn next_frame(&mut self) {
        loop {
            self.clock();
            if self.ppu.frame_completed() {
                break;
            }
        }
    }

    pub fn get_framebuffer(&self) -> &[u8] {
        &self.bus.lcd.framebuffer
    }

    pub fn clock(&mut self) {
        self.cpu.clock(&mut self.bus);
        self.bus.clock();
        self.ppu.clock(&mut self.bus);
    }

    pub fn plug_serial_device<SD: SerialDevice + 'static>(&mut self, serial_device: SD) -> Rc<RefCell<SD>> {
        self.bus.plug_serial_device(serial_device)
    } 

    pub fn remove_serial_device(&mut self) {
        self.bus.remove_serial_device();
    }
}