use crate::{cartridge::CartridgeInfo, constants::{CARTRIDGE_BANK_00_END, CARTRIDGE_BANK_00_START, CARTRIDGE_BANK_01NN_END, CARTRIDGE_BANK_01NN_START, EXTERNAL_RAM_END, EXTERNAL_RAM_START}, mbc::MBC};

pub struct MBC1 {
    enable_ram_register: bool,
    rom_bank_number: u8,
    ram_bank_number: u8,
    mode1: bool,
    ram: Vec<u8>,
    rom: Vec<u8>,
    cart_rom_banks: u8,
    cart_ram_banks: u8,
}

impl MBC1 {
    pub fn new(rom: Vec<u8>, cart_info: &CartridgeInfo) -> Self {
        Self {
            enable_ram_register: false,
            rom_bank_number: 1,
            ram_bank_number: 0,
            mode1: false,
            ram: vec![0; cart_info.ram_size as usize * 1024],
            rom,
            cart_rom_banks: cart_info.rom_banks,
            cart_ram_banks: cart_info.ram_banks,
        }
    }
}

impl MBC for MBC1 {
    fn read(&self, addr: u16) -> u8 {
        if (CARTRIDGE_BANK_00_START..=CARTRIDGE_BANK_00_END).contains(&addr) {
            self.rom[addr as usize]
        }
        else if (CARTRIDGE_BANK_01NN_START..=CARTRIDGE_BANK_01NN_END).contains(&addr) {
            let selected_rom_bank = (self.ram_bank_number << 5) | self.rom_bank_number;
            let addr = CARTRIDGE_BANK_01NN_START * selected_rom_bank as u16 + (addr - CARTRIDGE_BANK_01NN_START);
            self.rom[addr as usize]
        }
        else if (EXTERNAL_RAM_START..=EXTERNAL_RAM_END).contains(&addr) {
            let addr = self.ram_bank_number as u16 * 8 * 1024 + (addr - EXTERNAL_RAM_START);
            let addr = addr as usize;
            if self.ram.len() > addr {
                self.ram[addr]
            }
            else {
                0xFF
            }
        }
        else {
            0xFF
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        if (0x0000..=0x1FFF).contains(&addr) {
            self.enable_ram_register = (value & 0xF) == 0xA && self.ram.len() > 0;
        }
        else if (0x2000..=0x3FFF).contains(&addr) {
            self.rom_bank_number = value & 0x1F;
            if !self.mode1 {
                self.rom_bank_number = self.rom_bank_number.max(0x01);
            }
            let mask = !self.cart_rom_banks;
            self.rom_bank_number &= mask;    
        }
        else if (0x4000..=0x5FFF).contains(&addr) {
            if self.cart_rom_banks > 32 && self.cart_ram_banks > 1 {
                self.ram_bank_number = value & 0x3;
            }
        }
        else if (0x6000..=0x7FFF).contains(&addr) {
            self.mode1 = value & 1 > 0 && self.cart_rom_banks > 32 && self.cart_ram_banks > 1;
        }
        else if (EXTERNAL_RAM_START..=EXTERNAL_RAM_END).contains(&addr) {
            let addr = self.ram_bank_number as u16 * EXTERNAL_RAM_START + (addr - EXTERNAL_RAM_START);
            let addr = addr as usize;
            if self.ram.len() > addr {
                self.ram[addr] = value;
            }
        }
    }
}