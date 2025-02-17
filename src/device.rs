use std::ffi::CString;
use std::fmt::Display;

use arrayvec::ArrayVec;
use serialport::SerialPort;

use crate::shdlc::{MISOFrame, MOSIFrame, TranslationError};

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

    pub fn get_serial_number(&mut self) -> Result<String, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0xD0, &[0x03]);
        let data = frame.into_raw();

        let _ = self.port.write(&data).unwrap();
        let response = self.read_response()?;

        let parsed = MISOFrame::from_bytes(&response);

        let string = CString::from_vec_with_nul(parsed.into_data().to_vec());
        let string = match string {
            Ok(s) => match s.into_string() {
                Ok(st) => st,
                Err(_) => Err(DeviceError::InvalidString)?,
            },
            Err(_) => Err(DeviceError::InvalidString)?,
        };

        Ok(string)
    }

    pub fn get_article_code(&mut self) -> Result<String, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0xD0, &[0x02]);
        let _ = self.port.write(&frame.into_raw()).unwrap();
        let response = self.read_response()?;
        let parsed = MISOFrame::from_bytes(&response);
        let string = match CString::from_vec_with_nul(parsed.into_data().to_vec()) {
            Ok(s) => match s.into_string() {
                Ok(st) => st,
                Err(_) => Err(DeviceError::InvalidString)?,
            },
            Err(_) => Err(DeviceError::InvalidString)?,
        };

        Ok(string)

    }

    // for now test command to read device information
    pub fn read_response(&mut self) -> Result<ArrayVec<u8, 518>, DeviceError> {
        let mut buff = [0_u8; 20];
        let mut out = ArrayVec::<u8, 518>::new();
        loop {
            let s = self.port.read(&mut buff)?;
            out.try_extend_from_slice(&buff[..s]).unwrap();
            if buff[s-1] == 0x7E && (s > 1 || out.len() > 1) {
                break;
            }
        }

        let frame = MISOFrame::from_bytes(&out);
        if !frame.validate_checksum() {
            Err(DeviceError::InvalidChecksum(frame.get_checksum(), frame.calculate_check_sum()))?
        }

        Ok(out)
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

#[derive(Debug)]
pub enum DeviceError {
    IoError(std::io::Error),
    ShdlcError(TranslationError),
    InvalidChecksum(u8, u8),
    InvalidString,
}

impl Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => e.fmt(f),
            Self::ShdlcError(e) => e.fmt(f),
            Self::InvalidChecksum(recived, expected) => write!(f, "checksum recived: {:#02x} did not match expected value: {:#02x}", recived, expected),
            Self::InvalidString => write!(f, "invalid string data found")
        }
    }
}

impl From<std::io::Error> for DeviceError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<TranslationError> for DeviceError {
    fn from(value: TranslationError) -> Self {
        Self::ShdlcError(value)
    }
}
