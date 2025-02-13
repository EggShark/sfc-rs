use std::{thread, usize};

use arrayvec::ArrayVec;
use serialport::SerialPort;

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

    // for now test command to read device information
    pub fn send_message(&mut self) -> [u8; 256] {
        let mut ck: u8 = 0;
        ck = ck.wrapping_add(self.slave_adress);
        ck = ck.wrapping_add(0xD0_u8);
        ck = ck.wrapping_add(0x01);
        ck = ck.wrapping_add(0x03);
        ck ^= 0xFF_u8;
        let messgae: &[u8] = &[0x7E_u8, self.slave_adress, 0xD0_u8, 0x01_u8, 0x03_u8, ck, 0x7E_u8];

        let s = self.port.write(messgae).unwrap();
        println!("wrote {} bytes", s);

        let mut buff = [0_u8; 20];
        let mut out = [0_u8; 256];
        let mut idx = 0;
        loop {
            let s = self.port.read(&mut buff).unwrap();
            out[idx..(idx+s)].copy_from_slice(&buff[..s]);
            println!("{:#02x}, {}, {:?}", buff[s-1], s, &buff[..s]);
            if buff[s-1] == 0x7E && (s > 1 || idx > 0) {
                break;
            }
            idx += s;
        }
        let data_len = out[4] as usize;
        // for i in 5..5+data_len {
        //    out[i] = out[i].reverse_bits();
        // }
        let test = String::from_utf8_lossy(&out[5..5+data_len]);
        println!("{}", test);
        // let first = self.port.read_u8();
        // println!("{:?}", first)
        // let size = self.port.read(&mut out).unwrap();
        // println!("{:?}", out);
        // let mut out = [0_u8; 256];
        // let size = self.port.read(&mut out).unwrap();
        // println!("{:?}", out);
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
