
// Cartridge
pub const CARTRIDGE_BANK_00_START: u16 = 0x0000;
pub const CARTRIDGE_BANK_00_END: u16 = 0x3FFF;

pub const CARTRIDGE_BANK_01NN_START: u16 = 0x4000;
pub const CARTRIDGE_BANK_01NN_END: u16 = 0x7FFF;

// External RAM
pub const EXTERNAL_RAM_START: u16 = 0xA000;
pub const EXTERNAL_RAM_END: u16 = 0xBFFF;


// VRAM
pub const VRAM_MEM_START: u16 = 0x8000;
pub const VRAM_MEM_END: u16 = 0x9FFF;

// WRAM

pub const WRAM_MEM_START: u16 = 0xC000;
pub const WRAM_MEM_END: u16 = 0xCFFF;
pub const WRAM_BANK_MEM_START: u16 = 0xD000;
pub const WRAM_BANK_MEM_END: u16 = 0xDFFF;


// Echo RAM

pub const ECHO_RAM_START: u16 = 0xE000;
pub const ECHO_RAM_END: u16 = 0xFDFF;

// High RAM

pub const HRAM_MEM_START: u16 = 0xFF80;
pub const HRAM_MEM_END: u16 = 0xFFFE;


// IO Registers

pub const IO_INTERRUPTS: u16 = 0xFF0F;


// Interrupt Enable Register
pub const IE_REGISTER: u16 = 0xFFFF;