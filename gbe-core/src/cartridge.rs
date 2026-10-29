use crate::{constants::{CARTRIDGE_BANK_00_END, CARTRIDGE_BANK_00_START}, mcb::{MBC, RomMBC}};

pub struct Cartridge {
    data: Vec<u8>,
    mbc: Box<dyn MBC>,
}

impl Cartridge {

    pub fn from_memory(data: Vec<u8>) -> Self {

        let mbc = Self::resolve_mbc(&data);
        Self {
            data,
            mbc
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        if (CARTRIDGE_BANK_00_START..=CARTRIDGE_BANK_00_END).contains(&addr) {
            self.data[addr as usize]
        }
        else if let Some(addr) = self.mbc.resolve_address(addr) {
            self.data[addr as usize]
        }
        else {
            0xFF
        }
    }

    pub fn write_u8(&mut self, _addr: u16, _value: u8) {
        
    }

    fn resolve_mbc(data: &[u8]) -> Box<dyn MBC> {
        let cartridge_type = data[0x147];
        match cartridge_type {
            0x00 => Box::new(RomMBC),
            _ => unimplemented!("Unknown cartridge_type 0x{:02x}", cartridge_type)
        }
    }


}

#[cfg(test)]
impl Cartridge {
    pub(crate) fn dummy() -> Self {
        use crate::mcb::RomMBC;

        Self {
            data: vec![0xFF, 0xFF, 0x00],
            mbc: Box::new(RomMBC),
        }
    }
}