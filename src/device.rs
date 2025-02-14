use std::ffi::CString;

use arrayvec::ArrayVec;
use serialport::SerialPort;

use crate::shdlc::{MISOFrame, MOSIFrame};

pub struct Device<T: SerialPort> {
    port: T,
    pub last_error_flag: u32,
    pub slave_adress: u8,
}

impl<T: SerialPort> Device<T> {
    pub fn new(mut serial_port: T, slave_adress: u8) -> Self {
        let _ = serial_port.set_timeout(std::time::Duration::from_millis(400));

        Self {
            port: serial_port,
            last_error_flag: 0,
            slave_adress,
        }
    }

    pub fn get_serial_number(&mut self) -> String {
        let frame = MOSIFrame::new(self.slave_adress, 0xD0, &[0x03]);
        let data = frame.into_raw();

        let _ = self.port.write(&data).unwrap();
        let response = self.read_response();
        println!("{:?}", response);
        let parsed = MISOFrame::from_bytes(&response);
        println!("{:?}", parsed);
        let string = CString::from_vec_with_nul(parsed.into_data().to_vec()).unwrap().into_string().unwrap();
        println!("{:?}", string);

        string
    }

    // for now test command to read device information
    pub fn read_response(&mut self) -> ArrayVec<u8, 518> {
        let mut buff = [0_u8; 20];
        let mut out = ArrayVec::<u8, 518>::new();
        loop {
            let s = self.port.read(&mut buff).unwrap();
            out.try_extend_from_slice(&buff[..s]).unwrap();
            if buff[s-1] == 0x7E && (s > 1 || out.len() > 1) {
                break;
            }
        }

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
