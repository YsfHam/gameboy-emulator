pub struct Ram<const SIZE: usize> {
    data: [u8; SIZE],
}

impl<const SIZE: usize> Ram<SIZE> {
    pub fn new() -> Self {
        Self {
            data: [0; SIZE]
        }
    }

    pub fn len() -> usize {
        SIZE
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    pub fn write_u8(&mut self, addr: u16, value: u8) {
        self.data[addr as usize] = value;
    }
}

macro_rules! define_ram_type {
    ($size_name:ident, $ram_type:ident, $size:expr) => {
        struct $size_name {
            _private: ()
        }

        impl $size_name {
            const SIZE: usize = $size;
        }

        pub type $ram_type = Ram<{$size_name::SIZE}>;
    };
}

define_ram_type!(Mem8Kib, Ram8Kib, 8 * 1024);
define_ram_type!(Mem4Kib, Ram4Kib, 4 * 1024);