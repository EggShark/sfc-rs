use std::ffi::CStr;

use sfc_core::{error::DeviceError, shdlc::MISOFrame};

#[derive(Debug, PartialEq)]
pub struct CalibrationCondition {
    pub company: String,
    pub operator: String,
    pub calibration_year: u16,
    pub calibration_month: u8,
    pub calibration_day: u8,
    pub calibration_hour: u8,
    pub calibration_minute: u8,
    pub calibration_temperature: f32,
    pub calibration_inlet_temperature: f32,
    pub calibration_diffrential_pressure: f32,
    pub real_gas_calibration: bool,
    pub calibration_accuracy_setpoint: f32,
    pub calibration_accuracy_fullscale: f32,
}

impl CalibrationCondition {
    pub(crate) fn from_miso(frame: MISOFrame) -> Result<Self, DeviceError> {
        let data = frame.into_data();
        if data.len() < 127 {
            return Err(DeviceError::ShdlcError(sfc_core::shdlc::TranslationError::NotEnoughData(127, data.len() as u8)));
        }

        let company = match CStr::from_bytes_until_nul(&data[..50]) {
            Ok(s) => match s.to_str() {
                Ok(s) => s.to_string(),
                Err(_) => return Err(DeviceError::InvalidString),
            }
            Err(_) => return Err(DeviceError::InvalidString),
        };
        
        let operator = match CStr::from_bytes_until_nul(&data[50..100]) {
            Ok(s) => match s.to_str() {
                Ok(s) => s.to_string(),
                Err(_) => return Err(DeviceError::InvalidString),
            }
            Err(_) => return Err(DeviceError::InvalidString),
        };

        let calibration_year = u16::from_be_bytes([data[100], data[101]]);
        let calibration_month = data[102];
        let calibration_day = data[103];
        let calibration_hour = data[104];
        let calibration_minute = data[105];
        let calibration_temperature = f32::from_be_bytes([data[106], data[107], data[109], data[109]]);
        let calibration_inlet_temperature = f32::from_be_bytes([data[110], data[111], data[112], data[113]]);
        let calibration_diffrential_pressure = f32::from_be_bytes([data[114], data[115], data[116], data[117]]);
        let real_gas_calibration = data[118] > 0;
        let calibration_accuracy_setpoint = f32::from_be_bytes([data[119], data[120], data[121], data[122]]);
        let calibration_accuracy_fullscale = f32::from_be_bytes([data[123], data[124], data[125], data[126]]);

        Ok(Self {
            company,
            operator,
            calibration_year,
            calibration_month,
            calibration_day,
            calibration_hour,
            calibration_minute,
            calibration_temperature,
            calibration_inlet_temperature,
            calibration_diffrential_pressure,
            real_gas_calibration,
            calibration_accuracy_setpoint,
            calibration_accuracy_fullscale,
        })
    }
}
