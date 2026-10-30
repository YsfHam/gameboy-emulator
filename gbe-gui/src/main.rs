use std::{env::args, fs};

use gbe_core::{cartridge::Cartridge, gameboy::Gameboy, serial::ConsoleSerialDevice};

fn main() {
    let args = args().collect::<Vec<_>>();
    if args.len() < 2 {
        eprintln!("usage {} ROM_PATH", args[0]);
        return;
    }
    test_cpu(&args[1]);

}

fn test_cpu(filepath: &str) {
    let bytes = fs::read(filepath).unwrap();
    let cart = Cartridge::from_memory(bytes);
    let mut gameboy = Gameboy::new(cart);
    gameboy.plug_serial_device(ConsoleSerialDevice);

    loop {
        gameboy.clock();
    }
}
