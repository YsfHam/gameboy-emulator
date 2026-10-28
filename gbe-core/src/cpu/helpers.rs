pub fn add_u8(a: u8, b: u8, carry: bool) -> (u8, bool, bool) {
    let (result, new_carry) = a.carrying_add(b, carry);
    let half_carry = (a & 0xF) + (b & 0xF) + (carry as u8) > 0xF;
    (result, half_carry, new_carry)
}

pub fn add_u16(a: u16, b: u16) -> (u16, bool, bool) {
    let (result, carry) = a.overflowing_add(b);
    let half_carry = (a & 0xFFF) + (b & 0xFFF) > 0xFFF;
    (result, half_carry, carry)
}

pub fn sub_u8(a: u8, b: u8) -> (u8, bool, bool) {
    let (result, carry) = a.overflowing_sub(b);
    let half_carry = (a & 0xF) < (b & 0xF);
    (result, half_carry, carry)
}

pub fn carrying_sub_u8(a: u8, b: u8, carry: bool) -> (u8, bool, bool) {
    let sub = b as u16 + carry as u16;
    let a_u16 = a as u16;
    let result = (a_u16 - sub) as u8;
    let half_carry = (a & 0xF) < (sub & 0xF) as u8;
    let carry = a_u16 < sub;

    (result, half_carry, carry)
}

pub fn rotate_left(value: u8, carry: bool) -> (u8, bool) {
    let new_carry = value & 0x80 > 0;
    let result = (value << 1) | carry as u8;

    (result, new_carry)
}

pub fn circular_left_rotation(value: u8) -> (u8, bool) {
    let carry = value & 0x80 > 0;
    let result = value.rotate_left(1);
    (result, carry)
}

pub fn shift_left(value: u8) -> (u8, bool) {
    let new_carry = value & 0x80 > 0;
    let result = value << 1;

    (result, new_carry)
}

pub fn rotate_right(value: u8, carry: bool) -> (u8, bool) {
    let new_carry = value & 1 > 0;
    let result = value >> 1 | ((carry as u8) << 7);

    (result, new_carry)
}

pub fn circular_right_rotation(value: u8) -> (u8, bool) {
    let carry = value & 1 > 0;
    let result = value.rotate_right(1);
    (result, carry)
}

pub fn shift_right(value: u8) -> (u8, bool) {
    let carry = value & 1 > 0;
    let result = value >> 1;
    (result, carry)
}

pub fn arithmetic_shift_right(value: u8) -> (u8, bool) {
    let carry = value & 1 > 0;
    let result = value >> 1 | (value & (1 << 7));
    (result, carry)
}

pub fn swap_nibbles(value: u8) -> u8 {
    let hi = value >> 4;
    let lo = value & 0xF;

    (lo << 4) | hi
}
