pub trait SerialDevice {
    fn write(&mut self, data: u8);
}

pub struct ConsoleSerialDevice;

impl SerialDevice for ConsoleSerialDevice {
    fn write(&mut self, data: u8) {
        print!("{}", data as char);
    }
}