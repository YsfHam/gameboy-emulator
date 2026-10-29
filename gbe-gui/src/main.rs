use std::fs;

use gbe_core::{bus::Bus, cartridge::Cartridge, cpu::Cpu};

fn main() {
    let bytes = fs::read("C:\\dev\\gameboy-emulator\\roms\\cpu_instrs.gb").unwrap();
    let mut bus = Bus::new(Cartridge::from_memory(bytes));
    let mut cpu = Cpu::with_entry_point(0x100);


    loop {
        cpu.clock(&mut bus);
    }

}
