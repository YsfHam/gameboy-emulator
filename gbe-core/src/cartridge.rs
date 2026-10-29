use crate::mbc::{MBC, RomMBC, mbc1::MBC1};

#[derive(Debug, Clone)]
pub struct CartridgeInfo {
    pub title: String,
    pub cart_type: CartridgeType,
    pub rom_size: u32,
    pub rom_banks: u8,
    pub ram_size: u32,
    pub ram_banks: u8,
}

impl From<&[u8]> for CartridgeInfo {
    fn from(rom: &[u8]) -> Self {
        let title = Self::parse_title(rom);
        let cart_type = Self::parse_cartridge_type(rom);
        let (rom_size, rom_banks) = Self::parse_rom_size(rom);
        let (ram_size, ram_banks) = Self::parse_ram_size(rom);

        Self {
            title,
            cart_type,
            rom_size,
            rom_banks,
            ram_size,
            ram_banks,
        }
    }
}

impl CartridgeInfo {
    fn parse_title(rom: &[u8]) -> String {
        let title_bytes = &rom[0x134..=0x143];
        let mut i = 0;
        while title_bytes[i] != 0 {
            i = i + 1;
        }

        String::from_utf8(Vec::from(&title_bytes[..i])).unwrap()
    }

    fn parse_cartridge_type(rom: &[u8]) -> CartridgeType {
        CartridgeType::from(rom[0x147])
    }

    fn parse_rom_size(rom: &[u8]) -> (u32, u8) {
        let size = 32 * (1 << rom[0x148]);
        let banks_number = size / 16;

        (size, banks_number as u8)
    }

    fn parse_ram_size(rom: &[u8]) -> (u32, u8) {
        let ram_size = match rom[0x149] {
            0x00 | 0x01 => 0,
            0x02 => 8,
            0x03 => 32,
            0x04 => 128,
            0x05 => 64,
            value => unreachable!("Invalid value for ram size 0x{value:02x}")
        };

        let banks_number = ram_size / 8;

        (ram_size, banks_number as u8)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CartridgeType {
    Rom,
    MBC1,
}

impl From<u8> for CartridgeType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Rom,
            0x01 => Self::MBC1,
            _ => unimplemented!("Unknown value for Cartridge type 0x{value:02x}")
        }
    }
}


pub struct Cartridge {
    mbc: Box<dyn MBC>,
    pub info: CartridgeInfo,
}

impl Cartridge {

    pub fn from_memory(rom: Vec<u8>) -> Self {

        let info = CartridgeInfo::from(rom.as_slice());

        let mbc = Self::resolve_mbc(&info, rom);
        Self {
            mbc,
            info
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        self.mbc.read(addr)
    }

    pub fn write_u8(&mut self, addr: u16, value: u8) {
        self.mbc.write(addr, value);
    }

    fn resolve_mbc(info: &CartridgeInfo, rom: Vec<u8>) -> Box<dyn MBC> {
        match info.cart_type {
            CartridgeType::Rom => Box::new(RomMBC::new(rom)),
            CartridgeType::MBC1 => Box::new(MBC1::new(rom, info)),
        }
    }


}