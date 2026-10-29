use crate::constants::{CARTRIDGE_BANK_00_START, CARTRIDGE_BANK_01NN_END};


pub mod mbc1;

pub trait MBC {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}

pub struct RomMBC {
    rom: Vec<u8>,
}

impl RomMBC {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rom
        }
    }
}

impl MBC for RomMBC {
    fn read(&self, addr: u16) -> u8 {
        if (CARTRIDGE_BANK_00_START..=CARTRIDGE_BANK_01NN_END).contains(&addr) {
            self.rom[addr as usize]
        }
        else {
            0xFF
        }
    }

    fn write(&mut self, _: u16, _: u8) {

    }
}