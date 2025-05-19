use arrayvec::ArrayVec;
use serialport::SerialPort;

use sfc_core::gasunit::GasUnit;
use sfc_core::shdlc::{MISOFrame, MOSIFrame, TranslationError, Version};
use sfc_core::error::{DeviceError, StateResponseError};

use std::ffi::CString;

use crate::scaling::Scale;
use crate::valve_config::InputSourceConfig;
use crate::calibration::CalibrationCondition;

macro_rules! simple_device_function {
    ($name:ident, $ret_type:ty, $code:literal, $($data:literal),*) => {
       pub fn $name(&mut self) -> Result<$ret_type, DeviceError> {
           let frame = MOSIFrame::new(self.slave_address, $code, &[$($data,)*])?;
           let _ = self.port.write(&frame.into_raw())?;
           let data = self.read_response()?.into_data();

           if data.len() < std::mem::size_of::<$ret_type>() {
               return Err(DeviceError::ShdlcError(TranslationError::NotEnoughData(std::mem::size_of::<$ret_type>() as u8, data.len() as u8)));
           }

           let bytes_deffined: [u8; core::mem::size_of::<$ret_type>()] = core::array::from_fn(|i| data[i]);
           Ok(<$ret_type>::from_be_bytes(bytes_deffined))
       }
    };
}

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
        let frame = MOSIFrame::new(self.slave_address, 0x20, &[0x00, config.into()])?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        use InputSourceConfig::*;
        match config {
            Controller | ForceClosed | ForceOpen | Hold => Ok(()),
            UserDefined(value) => self.set_user_input_source(value),
        }
    }

    fn set_user_input_source(&mut self, value: f32) -> Result<(), DeviceError> {
        let value_b = value.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_address, 0x20, &[0x01, value_b[0], value_b[1], value_b[2], value_b[3]])?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    pub fn get_valve_input_source(&mut self) -> Result<InputSourceConfig, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x20, &[0x00])?;
        let _ =  self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();
        if data.is_empty() {
            Err(TranslationError::NotEnoughData(1, 0))?;
        }
        match data[0] {
            0x00 => Ok(InputSourceConfig::Controller),
            0x01 => Ok(InputSourceConfig::ForceClosed),
            0x02 => Ok(InputSourceConfig::ForceOpen),
            0x03 => Ok(InputSourceConfig::Hold),
            0x10 => self.get_user_input_value(),
            _ => unreachable!(),
        }
    }

    fn get_user_input_value(&mut self) -> Result<InputSourceConfig, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x20, &[0x01])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();
        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }
        let value = f32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        Ok(InputSourceConfig::UserDefined(value))
    }

    pub fn set_medium_unit_configuration(&mut self, unit: GasUnit) -> Result<(), DeviceError> {
       let frame = MOSIFrame::new(self.slave_address, 0x21, &[0x00, Into::<i8>::into(unit.unit_prefex).to_le_bytes()[0], unit.medium_unit.into(), unit.timebase.into()])?;
       let _ = self.port.write(&frame.into_raw())?;
       let _ = self.read_response()?;

       Ok(())
    }

    pub fn get_medium_unit_configuration(&mut self, include_wild_cards: bool) -> Result<GasUnit, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x21, &[include_wild_cards.into()])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 3 {
            return Err(DeviceError::ShdlcError(TranslationError::NotEnoughData(3, data.len() as u8)));
        }

        Ok(GasUnit {
            unit_prefex: i8::from_be_bytes([data[0]]).into(),
            medium_unit: data[1].into(),
            timebase: data[2].into(),
        })
    }

    pub fn get_converted_fullscale(&mut self) -> Result<f32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x21, &[0x0A])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();
        if data.len() < 4 {
            return Err(DeviceError::ShdlcError(TranslationError::NotEnoughData(4, data.len() as u8)));
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn set_user_controller_gain(&mut self, gain: f32) -> Result<(), DeviceError> {
        let gain_b = gain.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_address, 0x22, &[0x00, gain_b[0], gain_b[1], gain_b[2], gain_b[3]])?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    
    pub fn set_pressure_dependant_gain_enable(&mut self, enabled: bool) -> Result<(), DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x22, &[0x10, enabled.into()])?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    // inlet pressure is in bar
    pub fn set_gain_correction(&mut self, inlet_pressure: f32) -> Result<(), DeviceError> {
        let pressure_b = inlet_pressure.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_address, 0x22, &[0x11, pressure_b[0], pressure_b[1], pressure_b[2], pressure_b[3]])?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    pub fn set_gas_temperature_enable(&mut self, enabled: bool) -> Result<(), DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x22, &[0x20, enabled.into()])?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    pub fn set_inlet_temperature_correction(&mut self, temperature: f32) -> Result<(), DeviceError> {
        let temp_b = temperature.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_address, 0x22, &[0x21, temp_b[0], temp_b[1], temp_b[2], temp_b[3]])?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    pub fn get_user_controller_gain(&mut self) -> Result<f32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x22, &[0x00])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 4 {
            return Err(DeviceError::ShdlcError(TranslationError::NotEnoughData(4, data.len() as u8)));
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn get_pressure_dependant_gain(&mut self) -> Result<Option<f32>, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x22, &[0x10])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.is_empty() {
            return Err(DeviceError::ShdlcError(TranslationError::NotEnoughData(1, 0)));
        }

        if data[0] == 0 {
            return Ok(None);
        }

        let frame = MOSIFrame::new(self.slave_address, 0x022, &[0x11])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();
        if data.len() < 4 {
            return Err(DeviceError::ShdlcError(TranslationError::NotEnoughData(1, 0)));
        }
        
        Ok(Some(f32::from_be_bytes([data[0], data[1], data[2], data[3]])))
    }

    pub fn get_gas_temperature_compensation(&mut self) -> Result<Option<f32>, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x22, &[0x20])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.is_empty() {
            return Err(DeviceError::ShdlcError(TranslationError::NotEnoughData(1, 0)));
        }
        if data[0] == 0 {
            return Ok(None);
        }

        let frame = MOSIFrame::new(self.slave_address, 0x22, &[0x21])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 4 {
            return Err(DeviceError::ShdlcError(TranslationError::NotEnoughData(4, data.len() as u8)));
        }

        Ok(Some(f32::from_be_bytes([data[0], data[1], data[2], data[3]])))
    }

    pub fn measure_raw_flow(&mut self) -> Result<u16, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x30, &[0x00])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 2 {
            return Err(DeviceError::ShdlcError(TranslationError::NotEnoughData(2, data.len() as u8)));
        }

        Ok(u16::from_be_bytes([data[0], data[1]]))
    }
    
    pub fn measure_raw_thermal_conductivity(&mut self, valve_closed: bool) -> Result<u16, DeviceError> {
        let d1 = if valve_closed {0x01} else {0x02};
        let frame = MOSIFrame::new(self.slave_address, 0x30, &[d1])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 2 {
            return Err(DeviceError::ShdlcError(TranslationError::NotEnoughData(2, data.len() as u8)));
        }

        Ok(u16::from_be_bytes([data[0], data[1]]))
    }

    simple_device_function!{measure_temperature, f32, 0x30, 0x10}

    pub fn set_callibration(&mut self, index: u32) -> Result<(), DeviceError> {
        let index_b = index.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_address, 0x45, &index_b)?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    simple_device_function!(get_number_of_calibrations, u32, 0x40, 0x00);

    pub fn get_calibration_validity(&mut self, index: u32) -> Result<bool, DeviceError> {
        let index_b = index.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_address, 0x40, &[0x10, index_b[0], index_b[1], index_b[2], index_b[3]])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.is_empty() {
            return Err(DeviceError::ShdlcError(TranslationError::NotEnoughData(1, 0)))
        }

        Ok(data[0] > 0)
    }

    pub fn get_calibration_gas_description(&mut self, index: u32) -> Result<String, DeviceError> {
        let index_b = index.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_address, 0x40, &[0x11, index_b[0], index_b[1], index_b[2], index_b[3]])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data =  self.read_response()?.into_data();
        
        let string = match CString::from_vec_with_nul(data.to_vec()) {
            Ok(s) => match s.into_string() {
                Ok(st) => st,
                Err(_) => Err(DeviceError::InvalidString)?,
            },
            Err(_) => Err(DeviceError::InvalidString)?,
        };
        Ok(string)
    }

    pub fn get_calibration_gas_id(&mut self, index: u32) -> Result<u32, DeviceError> {
        let index_b = index.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_address, 0x40, &[0x12, index_b[0], index_b[1], index_b[2], index_b[3]])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 4 {
            return Err(DeviceError::ShdlcError(TranslationError::NotEnoughData(4, data.len() as u8)));
        }

        Ok(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn get_calibration_gas_unit(&mut self, index: u32) -> Result<GasUnit, DeviceError> {
        let index_b = index.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_address, 0x40, &[0x13, index_b[0], index_b[1], index_b[2], index_b[3]])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 3 {
            return Err(DeviceError::ShdlcError(TranslationError::NotEnoughData(3, data.len() as u8)));
        }

        Ok(GasUnit {
            unit_prefex: i8::from_be_bytes([data[0]]).into(),
            medium_unit: data[1].into(),
            timebase: data[2].into(),
        })
    }

    pub fn get_calibration_fullscale(&mut self, index: u32) -> Result<f32, DeviceError> {
        let index_b = index.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_address, 0x40, &[0x14, index_b[0], index_b[1], index_b[2], index_b[3]])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 4 {
            return Err(DeviceError::ShdlcError(TranslationError::NotEnoughData(4, data.len() as u8)));
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn get_calibration_initial_conditions(&mut self, index: u32) -> Result<CalibrationCondition, DeviceError> {
        let index_b = index.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_address, 0x40, &[0x15, index_b[0], index_b[1], index_b[2], index_b[3]])?;
        let _ = self.port.write(&frame.into_raw())?;
        let res_frame = self.read_response()?;

        CalibrationCondition::from_miso(res_frame)
    }

    pub fn get_calibration_recalibration_conditions(&mut self, index: u32) -> Result<CalibrationCondition, DeviceError> {
        let index_b = index.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_address, 0x40, &[0x16, index_b[0], index_b[1], index_b[2], index_b[3]])?;
        let _ = self.port.write(&frame.into_raw())?;
        let res_frame = self.read_response()?;

        CalibrationCondition::from_miso(res_frame)
    }

    pub fn get_calibration_thermal_conductivity_refrence(&mut self, index: u32) -> Result<u16, DeviceError> {
        let index_b = index.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_address, 0x40, &[0x16, index_b[0], index_b[1], index_b[2], index_b[3]])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 2 {
            return Err(DeviceError::ShdlcError(TranslationError::NotEnoughData(2, data.len() as u8)));
        }

        Ok(u16::from_be_bytes([data[0], data[1]]))
    }

    pub fn get_current_gas_description(&mut self) -> Result<String, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x44, &[0x11])?;
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

    simple_device_function!(get_current_gas_id, u32, 0x44, 0x12);
    simple_device_function!(get_current_gas_unit, GasUnit, 0x44, 0x13);
    simple_device_function!(get_current_fullscale, f32, 0x44, 0x14);

    pub fn get_current_initial_calibration_conditions(&mut self) -> Result<CalibrationCondition, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x44, &[0x15])?;
        let _ = self.port.write(&frame.into_raw());
        let res_frame = self.read_response()?;

        CalibrationCondition::from_miso(res_frame)
    }

    pub fn get_current_recalibration_condition(&mut self) -> Result<CalibrationCondition, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x44, &[0x16])?;
        let _ = self.port.write(&frame.into_raw());
        let res_frame = self.read_response()?;

        CalibrationCondition::from_miso(res_frame)
    }

    simple_device_function!(get_current_thermal_conducitvity_refrence, u16, 0x44, 0x17);

    pub fn read_user_memory(&mut self, start_address: u8, bytes_to_read: u8) -> Result<Vec<u8>, DeviceError> {
        let frame = MOSIFrame::new(self.slave_address, 0x6E, &[start_address, bytes_to_read])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        Ok(data.to_vec())
    }

    pub fn write_user_memory(&mut self, start_address: u8, data: &[u8]) -> Result<(), DeviceError> {
        let len = data.len() as u8;
        let mut  frame_data = vec![start_address, len];
        frame_data.extend_from_slice(data);
        let frame = MOSIFrame::new(self.slave_address, 0x6E, &frame_data)?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;

        Ok(())
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
