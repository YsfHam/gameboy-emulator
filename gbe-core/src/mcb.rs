use crate::constants::{CARTRIDGE_BANK_01NN_END, CARTRIDGE_BANK_01NN_START};

pub trait MBC {
    fn resolve_address(&self, addr: u16) -> Option<u16>;
}

pub struct RomMBC;

impl MBC for RomMBC {
    fn resolve_address(&self, addr: u16) -> Option<u16> {
        (CARTRIDGE_BANK_01NN_START..=CARTRIDGE_BANK_01NN_END)
            .contains(&addr)
            .then(|| addr)
    }
}