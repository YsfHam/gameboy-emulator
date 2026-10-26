pub fn add_u8(a: u8, b: u8, carry: bool) -> (u8, bool, bool) {
    let (result, carry) = a.carrying_add(b, carry);
    let half_carry = (a & 0xF) + (b & 0xF) + (carry as u8) > 0xF;
    (result, half_carry, carry)
}

pub fn add_u16(a: u16, b: u16) -> (u16, bool, bool) {
    let (result, carry) = a.overflowing_add(b);
    let half_carry = (a & 0xFFF) + (b & 0xFFF) > 0xFFF;
    (result, half_carry, carry)
}