use std::fs;

use gbe_core::{bus::Bus, cartridge::Cartridge, cpu::Cpu};

fn main() {
    test_cpu("C:\\dev\\gameboy-emulator\\roms\\cpu_instrs.gb");

}

fn test_cpu(filepath: &str) {
    let bytes = fs::read(filepath).unwrap();
    let cart = Cartridge::from_memory(bytes);
    let mut bus = Bus::new(cart);
    let mut cpu = Cpu::with_entry_point(0x100);


    loop {
        cpu.clock(&mut bus);
    }
}
