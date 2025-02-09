use arrayvec::ArrayVec;
use serialport::SerialPort;

pub struct Device<T: SerialPort> {
    port: T,
    pub last_error_flag: u32,
    pub slave_adress: u8,
}

impl<T: SerialPort> Device<T> {
    pub fn new(mut serial_port: T, slave_adress: u8) -> Self {
        let _ = serial_port.set_timeout(std::time::Duration::from_millis(200));

        Self {
            port: serial_port,
            last_error_flag: 0,
            slave_adress,
        }
    }

    // for now test command to read device information
    fn send_message(&mut self) -> ArrayVec<u8, 261> {
        let mut ck: u8 = 0;
        ck = ck.wrapping_add(self.slave_adress);
        ck = ck.wrapping_add(0xD0);
        ck = ck.wrapping_add(0x01);
        ck = ck.wrapping_add(0x01);
        ck ^= 0xFF;
        let messgae = [0x7E, self.slave_adress, 0xD0, 0x01, ck, 0x7E];

        let _ = self.port.write(&messgae).unwrap();
        let mut out = ArrayVec::new();
        
        let size = self.port.read(&mut out).unwrap();
        assert_eq!(size, out.len());
        out
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
