use std::{env::args, fs};

use gbe_core::{cartridge::Cartridge, constants::{SCREEN_HEIGHT, SCREEN_WIDTH}, gameboy::Gameboy, serial::ConsoleSerialDevice};
use minifb::{Key, Window, WindowOptions};
const SCALE: usize = 4;
const WIDTH: usize = SCREEN_WIDTH as usize * SCALE;
const HEIGHT: usize = SCREEN_HEIGHT as usize * SCALE;

fn main() {
    let args = args().collect::<Vec<_>>();
    if args.len() < 2 {
        eprintln!("usage {} ROM_PATH", args[0]);
        return;
    }
    let filepath = &args[1];
    let bytes = fs::read(filepath).unwrap();
    let cart = Cartridge::from_memory(bytes);
    let mut gameboy = Gameboy::new(cart);
    gameboy.plug_serial_device(ConsoleSerialDevice);

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {

        gameboy.next_frame();
        let framebuffer = gameboy.get_framebuffer();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let fb = framebuffer[(y / SCALE) * SCREEN_WIDTH as usize + x / SCALE];
                buffer[y * WIDTH + x] = match fb {
                    0 => 0x9bbc0f,
                    1 => 0x8bac0f,
                    2 => 0x306230,
                    3 => 0x0f380f,
                    _ => 0xff00ff, // magenta = unwritten pixel
                };
        }
}

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }

}
