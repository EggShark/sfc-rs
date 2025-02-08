use serialport::SerialPort;

pub struct Device<T: SerialPort> {
    port: T,
    pub last_error_flag: u32,
    pub slave_adress: u32,
}

impl<T: SerialPort> Device<T> {
    pub fn new(mut serial_port: T, slave_adress: u32) -> Self {
        let _ = serial_port.set_timeout(std::time::Duration::from_millis(200));

        Self {
            port: serial_port,
            last_error_flag: 0,
            slave_adress,
        }
    }
}

#[repr(u8)]
pub enum Scalling {
    Normailized,
    Physical,
    UserDefined,
}

#[repr(u8)]
pub enum ValveInputSource {
    Controller,
    ForceClosed,
    ForceOpen,
    Hold,
    UserDefined = 16,
}
