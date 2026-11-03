use std::{cell::RefCell, rc::Rc};

use crate::{
    bus::Bus,
    cartridge::Cartridge,
    cpu::Cpu,
    events::{DEFAULT_EVENT_BUFFER_CAPACITY, Event, EventBuffer, EventFilter},
    ppu::Ppu,
    serial::SerialDevice,
};



pub struct Gameboy {
    cpu: Cpu,
    bus: Bus,
    ppu: Ppu,
    cycle: u64,
    events: EventBuffer,
}

impl Gameboy {
    pub fn new(cartridge: Cartridge) -> Self {
        Self::with_event_capacity(cartridge, DEFAULT_EVENT_BUFFER_CAPACITY)
    }

    pub fn with_event_capacity(cartridge: Cartridge, event_capacity: usize) -> Self {
        Self {
            cpu: Cpu::with_entry_point(0x100),
            bus: Bus::new(cartridge),
            ppu: Ppu::new(),
            cycle: 0,
            events: EventBuffer::new(event_capacity, EventFilter::default()),
        }
    }

    pub fn set_event_filter(&mut self, filter: EventFilter) {
        self.events.set_filter(filter);
    }

    pub fn drain_events(&mut self) -> Vec<Event> {
        self.events.drain()
    }

    pub fn dropped_events(&self) -> u64 {
        self.events.dropped_events()
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
        let cycle = self.cycle;
        self.cpu.clock(&mut self.bus, &mut self.events, cycle);
        self.bus.clock(&mut self.events, cycle);
        self.ppu.clock(&mut self.bus, &mut self.events, cycle);
        self.cycle = self.cycle.wrapping_add(1);
    }

    pub fn plug_serial_device<SD: SerialDevice + 'static>(&mut self, serial_device: SD) -> Rc<RefCell<SD>> {
        self.bus.plug_serial_device(serial_device)
    } 

    pub fn remove_serial_device(&mut self) {
        self.bus.remove_serial_device();
    }
}