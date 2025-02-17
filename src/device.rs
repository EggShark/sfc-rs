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

    pub fn get_baudrate(&mut self) -> Result<u32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x91, &[]);
        let _ = self.port.write(&frame.into_raw())?;

        let response = self.read_response()?;
        let data = response.into_data();
        if data.len() < 4 {
            panic!("WOOOW not enough bytes");
        }
        let res = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);

        Ok(res)
    }

    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x91, &baudrate.to_be_bytes());
        let _ = self.port.write(&frame.into_raw())?;
        let res = self.read_response()?;
        if !res.is_ok() {
            println!("{:#010b}", res.get_state());
            Err(DeviceError::InvalidDataSent)?;
        }

        Ok(())
    }

    pub fn get_product_type(&mut self) -> Result<String, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0xD0, &[0x00]);
        let _ = self.port.write(&frame.into_raw())?;

        let response = self.read_response()?;
        let string = match CString::from_vec_with_nul(response.into_data().to_vec()) {
            Ok(s) => match s.into_string() {
                Ok(st) => st,
                Err(_) => Err(DeviceError::InvalidString)?,
            },
            Err(_) => Err(DeviceError::InvalidString)?,
        };

        Ok(string)
    }

    pub fn get_product_name(&mut self) -> Result<String, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0xD0, &[0x01]);
        let _ = self.port.write(&frame.into_raw())?;
        let response = self.read_response()?;
        let string = match CString::from_vec_with_nul(response.into_data().to_vec()) {
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
        let _ = self.port.write(&frame.into_raw())?;
        let response = self.read_response()?;
        let string = match CString::from_vec_with_nul(response.into_data().to_vec()) {
            Ok(s) => match s.into_string() {
                Ok(st) => st,
                Err(_) => Err(DeviceError::InvalidString)?,
            },
            Err(_) => Err(DeviceError::InvalidString)?,
        };

        Ok(string)

    }

    pub fn get_serial_number(&mut self) -> Result<String, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0xD0, &[0x03]);
        let data = frame.into_raw();

        let _ = self.port.write(&data)?;
        let response = self.read_response()?;

        let string = CString::from_vec_with_nul(response.into_data().to_vec());
        let string = match string {
            Ok(s) => match s.into_string() {
                Ok(st) => st,
                Err(_) => Err(DeviceError::InvalidString)?,
            },
            Err(_) => Err(DeviceError::InvalidString)?,
        };

        Ok(string)
    }

    

    // for now test command to read device information
    pub fn read_response(&mut self) -> Result<MISOFrame, DeviceError> {
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

        Ok(frame)
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
    InvalidDataSent,
}

impl Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => e.fmt(f),
            Self::ShdlcError(e) => e.fmt(f),
            Self::InvalidChecksum(recived, expected) => write!(f, "checksum recived: {:#02x} did not match expected value: {:#02x}", recived, expected),
            Self::InvalidString => write!(f, "invalid string data found"),
            Self::InvalidDataSent => write!(f, "the data sent to the device is not allowed for that command"),
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


#[cfg(test)]
mod tests {
    use serial_test::serial;

    #[cfg(target_os="linux")]
    use serialport::TTYPort;
    #[cfg(target_os="windows")]
    use serialport::COMPort;

    
    #[cfg(target_os="linux")]
    const PORT: &str = "/dev/ttyUSB0";
    #[cfg(target_os="windows")]
    const PORT: &str = "COM4";

    use super::*;

    #[cfg(target_os="linux")]
    type SP = TTYPort;

    fn create_device() -> Device<SP> {
        let test_port = serialport::new(PORT, 115200).open_native().unwrap();
        Device::new(test_port, 0)
    }

    #[test]
    #[serial]
    fn product_type() {
        let mut device = create_device();
        let pt = device.get_product_type().unwrap();
        println!("Product type: {}", pt);
    }

    #[test]
    #[serial]
    fn product_name() {
        let mut device = create_device();
        let pn = device.get_product_name().unwrap();
        println!("Product name: {}", pn);
    }

    #[test]
    #[serial]
    fn article_code() {
        let mut device = create_device();
        let ac = device.get_article_code().unwrap();
        println!("Article code: {}", ac);
    }

    #[test]
    #[serial]
    fn serial_number() {
        let mut device = create_device();
        let sn = device.get_serial_number().unwrap();
        println!("Serial number: {}", sn);
    }

    #[test]
    #[serial]
    fn get_baudrate() {
        let mut device = create_device();
        let br = device.get_baudrate().unwrap();
        assert_eq!(br, 115200);
    }

    #[test]
    #[serial]
    fn set_baudrate() {
        let mut device = create_device();
        device.set_baudrate(115200).unwrap();
    }

    #[test]
    #[serial]
    fn set_and_read_buadrate() {
        let mut device = create_device();
        device.set_baudrate(57600).unwrap();
        drop(device);
        let test_port = serialport::new(PORT, 57600).open_native().unwrap();
        let mut device = Device::new(test_port, 0);
        let br = device.get_baudrate().unwrap();
        device.set_baudrate(115200).unwrap();
        assert_eq!(br, 57600);
    }

    #[test]
    #[serial]
    fn set_invalid_buadrate() {
        let mut device = create_device();
        let res = device.set_baudrate(57601);
        match res {
            Err(DeviceError::InvalidDataSent) => {},
            _ => panic!("expected, DeviceError::InvalidDataSent"),
        }
    }
}
