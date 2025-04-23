use arrayvec::ArrayVec;
use serialport::SerialPort;

use sfc_core::shdlc::{MISOFrame, MOSIFrame, TranslationError, Version};
use sfc_core::error::{DeviceError, StateResponseError};

use std::ffi::CString;

use crate::scaling::Scale;
use crate::valve_config::InputSourceConfig;

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

    pub fn get_version(&mut self) -> Result<Version, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0xD1, &[])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();
        if data.len() < 7 {
            Err(TranslationError::NotEnoughData(7, data.len() as u8))?;
        }

        Ok(Version {
            firmware_major: data[0],
            firmware_minor: data[1],
            debug: data[2] > 0,
            hardware_major: data[3],
            hardware_minor: data[4],
            protocol_major: data[5],
            protocol_minor: data[6],
        })
    }

    // TODO: make this more rusty
    pub fn get_device_error_state(&mut self, clear_after_read: bool) -> Result<(u32, u8), DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0xD2, &[clear_after_read as u8])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();
        if data.len() < 5 {
            Err(TranslationError::NotEnoughData(5, data.len() as u8))?;
        }

        let code = u32::from_be_bytes([data[0],data[1],data[2],data[3]]);
        Ok((code, data[4]))
    }

    pub fn set_slave_address(&mut self, new_addres: u8) -> Result<(), DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x90, &[new_addres])?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    pub fn get_device_address(&mut self) -> Result<u8, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x90, &[])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();
        if data.is_empty() {
            Err(TranslationError::NotEnoughData(0, 1))?;
        }
        Ok(data[0])  
    }

    pub fn set_baudrate(&mut self, buad_rate: u32) -> Result<(), DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x91, &buad_rate.to_be_bytes())?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    pub fn get_baudrate(&mut self) -> Result<u32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x91, &[])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();
        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }
        Ok(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn reset_device(&mut self) -> Result<(), DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0xD3, &[])?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    pub fn factory_reset(&mut self) -> Result<(), DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x92, &[])?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    pub fn set_setpoint(&mut self, setpoint: u32, scale: Scale) -> Result<(), DeviceError> {
        let setpoint_bytes = setpoint.to_be_bytes();
        let frame = MOSIFrame::new(
            self.slave_address,
            0x00,
            &[
                scale as u8,
                setpoint_bytes[0],
                setpoint_bytes[1],
                setpoint_bytes[2],
                setpoint_bytes[3],
            ],
        )?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    pub fn get_setpoint(&mut self, scale: Scale) -> Result<u32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x00, &[scale as u8])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();
        
        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(u32::from_be_bytes([data[0],data[1],data[2],data[3]]))
    }

    pub fn read_measured_flow(&mut self, scale: Scale) -> Result<u32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x08, &[scale as u8])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();
        
        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(u32::from_be_bytes([data[0],data[1],data[2],data[3]]))
    }

    pub fn read_measured_flow_buffered(&mut self, scale: Scale) -> Result<BufferedRead, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x09, &[scale as u8])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();
        
        if data.len() < 12 {
            Err(TranslationError::NotEnoughData(12, data.len() as u8))?;
        }

        Ok(BufferedRead::new(&data))
    }

    /// TODO: make feature flag for V1.48
    pub fn read_measured_flow_two_sensors(&mut self, scale: Scale) -> Result<(f32, f32), DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x0A, &[scale as u8])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 8 {
            Err(TranslationError::NotEnoughData(8, data.len() as u8))?;
        }
        let sensor_1_data = f32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let sensor_2_data = f32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        Ok((sensor_1_data, sensor_2_data))
    }

    pub fn set_setpoint_and_read_measured_value(&mut self, scale: Scale, setpoint: f32) -> Result<f32, DeviceError> {
        let setpoint_bytes = setpoint.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_address, 0x03, &[scale as u8, setpoint_bytes[0], setpoint_bytes[1], setpoint_bytes[2], setpoint_bytes[3]])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    /// TODO: make feature flag for V1.48
    pub fn set_setpoint_and_read_measured_value_two_sensors(&mut self, scale: Scale, setpoint: f32) -> Result<(f32, f32), DeviceError> {
        let setpoint_bytes = setpoint.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_address, 0x04, &[scale as u8, setpoint_bytes[0], setpoint_bytes[1], setpoint_bytes[2], setpoint_bytes[3]])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 8 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        let sensor_1_data = f32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let sensor_2_data = f32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        Ok((sensor_1_data, sensor_2_data))
    }

    pub fn make_setpoint_persistant(&mut self, persist: bool) -> Result<(), DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x02, &[0x00, persist as u8])?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        
        Ok(())
    }

    pub fn is_setpoint_persistant(&mut self) -> Result<bool, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x02, &[0x00])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();
        
        if data.is_empty() {
            Err(TranslationError::NotEnoughData(1, 0))?;
        }

        Ok(data[0] == 1)
    }

    pub fn set_valve_input_source(&mut self, config: InputSourceConfig) -> Result<(), DeviceError> {
        let frame = match config {
            InputSourceConfig::Hold | InputSourceConfig::Controller | InputSourceConfig::ForceOpen | InputSourceConfig::ForceClosed 
                => MOSIFrame::new(self.slave_address, 0x20, &[0, config.into()])?,
            InputSourceConfig::UserDefined(v) => {
                let data_bytes = v.to_be_bytes();
                MOSIFrame::new(self.slave_address, 0x020, &[1, data_bytes[0], data_bytes[1], data_bytes[2], data_bytes[3]])?
            }
        };
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;

        Ok(())
    }

    pub fn get_valve_input_source(&mut self) -> Result<InputSourceConfig, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x20, &[])?;
        todo!()
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

#[derive(Debug, PartialEq)]
pub struct BufferedRead {
    pub lost_values: u32,
    pub remaning_values: u32,
    pub sampling_time: f32,
    pub values: ArrayVec<f32, 60>,
}

impl BufferedRead {
    /// assumes data_len has been checked
    pub(crate) fn new(data: &[u8]) -> Self {
        let lost_values = u32::from_be_bytes([data[0],data[1],data[2],data[3]]);
        let remaning_values = u32::from_be_bytes([data[4],data[5],data[6],data[7]]);
        let sampling_time =  f32::from_be_bytes([data[8],data[9], data[10], data[11]]);
        let mut values = ArrayVec::new();
        for chunk in data[12..].chunks(4) {
           if chunk.len() < 4 || values.len() == 60 {
               break;
           }
           values.push(f32::from_be_bytes([chunk[0],chunk[1],chunk[2],chunk[3]]));
        }
        Self {
            lost_values,
            remaning_values,
            sampling_time,
            values
        }
    }
}
