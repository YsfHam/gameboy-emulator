use std::fs;

use gbe_core::{cartridge::Cartridge, gameboy::Gameboy};

fn main() {
    test_cpu("C:\\dev\\gameboy-emulator\\roms\\instr_timing.gb");

}

fn test_cpu(filepath: &str) {
    let bytes = fs::read(filepath).unwrap();
    let cart = Cartridge::from_memory(bytes);
    let mut gameboy = Gameboy::new(cart);


    loop {
        gameboy.clock();
    }
}
