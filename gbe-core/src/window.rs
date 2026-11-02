pub struct Window {
    pub wy: u8,
    pub wx: u8
}

impl Window {
    pub fn new() -> Self {
        Self {
            wx: 7,
            wy: 0
        }
    }
}