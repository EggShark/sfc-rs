use arrayvec::ArrayVec;
use serialport::SerialPort;

use sfc_core::shdlc::{MOSIFrame, MISOFrame};
use sfc_core::error::{DeviceError, StateResponseError};

use std::ffi::CString;

pub struct Device<T: SerialPort> {
    port: T,
    slave_address: u8,
}

pub struct DeviceInformation;

impl<T: SerialPort> Device<T> {
    pub fn new(port: T, slave_address: u8) -> Result<Self, DeviceError> {
        
        Ok(Self {
            port,
            slave_address,
        })
    }

    pub fn get_product_name(&mut self) -> Result<String, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0xD0, &[0x01])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();
        let string = match CString::from_vec_with_nul(data.to_vec()) {
            Ok(s) => match s.into_string() {
                Ok(st) => st,
                Err(_) => Err(DeviceError::InvalidString)?,
            },
            Err(_) => Err(DeviceError::InvalidString)?,
        };
        Ok(string)
    }

    pub fn get_article_code(&mut self) -> Result<String, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0xD0, &[0x02])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();
        let string = match CString::from_vec_with_nul(data.to_vec()) {
            Ok(s) => match s.into_string() {
                Ok(st) => st,
                Err(_) => Err(DeviceError::InvalidString)?,
            },
            Err(_) => Err(DeviceError::InvalidString)?,
        };
        Ok(string)
    }

    pub fn get_serial_number(&mut self) -> Result<String, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0xD0, &[0x03])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();
        let string = match CString::from_vec_with_nul(data.to_vec()) {
            Ok(s) => match s.into_string() {
                Ok(st) => st,
                Err(_) => Err(DeviceError::InvalidString)?,
            },
            Err(_) => Err(DeviceError::InvalidString)?,
        };
        Ok(string)

    }

    fn read_response(&mut self) -> Result<MISOFrame, DeviceError> {
        let mut buff = [0_u8; 20];
        let mut out = ArrayVec::<u8, 518>::new();
        loop {
            let s = self.port.read(&mut buff)?;
            out.try_extend_from_slice(&buff[..s])?;
            if buff[s - 1] == 0x7E && (s > 1 || out.len() > 1) {
                break;
            }
        }

        let frame = MISOFrame::from_bytes(&out);

        if !frame.is_ok() {
            Err(StateResponseError::from(frame.get_state()))?;
        }

        if !frame.validate_checksum() {
            Err(DeviceError::InvalidChecksum(
                frame.get_checksum(),
                frame.calculate_check_sum(),
            ))?;
        }

        Ok(frame)
    }   
}

