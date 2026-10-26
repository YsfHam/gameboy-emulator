#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Register8 {
    A = 0, F, B, C, D, E, H, L
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Register16 {
    AF = 0,
    BC,
    DE,
    HL,
    SP,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Flag {
    Carry = 4,
    HalfCarry,
    Substraction,
    Zero
}


pub struct Registers {
    values: [Register; 5],
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            values: [Register::new(); 5]
        }
    }
}

impl Registers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_u16(&self, reg: Register16) -> u16 {
        self.values[reg as usize].value
    }

    pub fn write_u16(&mut self, reg: Register16, value: u16) {
        self.values[reg as usize].value = value;
    }

    pub fn read_u8(&self, reg: Register8) -> u8 {
        let reg_index = (reg as usize) / 2;
        let register = &self.values[reg_index];
        register.get((reg as usize) % 2).unwrap()
    }

    pub fn write_u8(&mut self, reg: Register8, value: u8) {
        let reg_index = (reg as usize) / 2;
        let register = &mut self.values[reg_index];
        register.set((reg as usize) % 2, value);
    }

    pub fn set_flag(&mut self, flag: Flag) {
        let reg_f = self.read_u8(Register8::F);
        let new_reg_f = reg_f | (1 << flag as u8);
        self.write_u8(Register8::F, new_reg_f);
    }

    pub fn unset_flag(&mut self, flag: Flag) {
        let reg_f = self.read_u8(Register8::F);
        let new_reg_f = reg_f & !(1 << flag as u8);
        self.write_u8(Register8::F, new_reg_f);
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        let reg_f = self.read_u8(Register8::F);

        (reg_f >> flag as u8) & 1 > 0
    }

    pub fn update_flag(&mut self, flag: Flag, value: bool) {
        if value {
            self.set_flag(flag);
        }
        else {
            self.unset_flag(flag);
        }
    }
}


#[derive(Clone, Copy)]
#[repr(transparent)]
struct Register {
    value: u16,
}

impl Default for Register {
    fn default() -> Self {
        Self { value: 0 }
    }
}

impl Register {
    fn new() -> Self {
        Self::default()
    }

    fn hi(&self) -> u8 {
        (self.value >> 8) as u8
    }

    fn lo(&self) -> u8 {
        (self.value & 0xFF) as u8
    }
    
    fn set_hi(&mut self, hi: u8) {
        self.value = ((hi as u16) << 8) | self.lo() as u16;
    }

    fn set_lo(&mut self, lo: u8) {
        self.value = (self.value & 0xFF00) | (lo as u16)
    }

    fn get(&self, index: usize) -> Option<u8> {
        match index {
            0 => Some(self.hi()),
            1 => Some(self.lo()),
            _ => None
        }
    }

    fn set(&mut self, index: usize, value: u8) {
        match index {
            0 => self.set_hi(value),
            1 => self.set_lo(value),
            _ => panic!("Index out of bound! index: {}, len: 2", index)

        }
    }
}