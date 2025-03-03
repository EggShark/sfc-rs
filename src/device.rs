use std::ffi::CString;
use std::fmt::Display;

use arrayvec::{ArrayVec, CapacityError};
use serialport::SerialPort;

use crate::{gasunit::{GasUnit, Prefixes, TimeBases, Units}, shdlc::{MISOFrame, MOSIFrame, TranslationError}};

pub struct Device<T: SerialPort> {
    port: T,
    pub last_error_flag: u32,
    pub slave_adress: u8,
}

impl<T: SerialPort> Device<T> {
    pub fn new(mut serial_port: T, slave_adress: u8) -> Self {
        serial_port.set_timeout(std::time::Duration::from_millis(600)).unwrap();

        Self {
            port: serial_port,
            last_error_flag: 0,
            slave_adress,
        }
    }

    pub fn get_setpoint(&mut self) -> Result<f32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x00, &[0x01]);
        let _ = self.port.write(&frame.into_raw())?;
        let res = self.read_response()?;
        let data = res.into_data();
        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn set_setpoint(&mut self, setpoint: f32) -> Result<(), DeviceError> {
        let setpoint_bytes = setpoint.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_adress, 0x00, &[0x01, setpoint_bytes[0], setpoint_bytes[1], setpoint_bytes[2], setpoint_bytes[3]]);

        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    pub fn read_measured_value(&mut self) -> Result<f32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x08, &[0x01]);
        let _ = self.port.write(&frame.into_raw())?;
        let res = self.read_response()?;
        let data = res.into_data();
        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(f32::from_be_bytes([data[0],data[1],data[2],data[3]]))
    }

    pub fn read_average_measured_value(&mut self, measurment_count: u8) -> Result<f32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x08, &[0x11, measurment_count]);
        let raw = frame.into_raw();

        let _ = self.port.write(&raw)?;
        let res = self.read_response()?;
        let data = res.into_data();
        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn set_setpoint_and_read_measured_value(&mut self, setpoint: f32) -> Result<f32, DeviceError> {
        let setpoint_bytes = setpoint.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_adress, 0x03, &[0x01, setpoint_bytes[0], setpoint_bytes[1], setpoint_bytes[2], setpoint_bytes[3]]);
        let _ = self.port.write(&frame.into_raw())?;
        let res = self.read_response()?;
        let data = res.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn get_controller_gain(&mut self) -> Result<f32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x22, &[0x00]);
        let _ = self.port.write(&frame.into_raw())?;
        let res = self.read_response()?;
        let data = res.into_data();

        
        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }
        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn set_controller_gain(&mut self, gain: f32) -> Result<(), DeviceError> {
        let gain_bytes = gain.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_adress, 0x22, &[0x00, gain_bytes[0], gain_bytes[1], gain_bytes[2], gain_bytes[3]]);
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    pub fn get_initial_step(&mut self) -> Result<f32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x22, &[0x03]);
        let _ = self.port.write(&frame.into_raw())?;
        let res = self.read_response()?;
        let data = res.into_data();
        
        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }
        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn set_initial_step(&mut self, step: f32) -> Result<(), DeviceError> {
        let step_bytes = step.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_adress, 0x22, &[0x03, step_bytes[0], step_bytes[1], step_bytes[2], step_bytes[3]]);
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    pub fn measure_raw_flow(&mut self) -> Result<u16, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x30, &[0x00]);
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 2 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(u16::from_be_bytes([data[0], data[1]]))
    }

    pub fn measure_raw_thermal_conductivity(&mut self) -> Result<u16, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x30, &[0x02]);
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 2 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(u16::from_be_bytes([data[0], data[1]]))
    }

    pub fn measure_temperature(&mut self) -> Result<f32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x30, &[0x10]);
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn get_number_of_calibrations(&mut self) -> Result<u32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x40, &[0x00]);
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }
        Ok(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn get_calibration_validity(&mut self, calibration_index: u32) -> Result<bool, DeviceError> {
        let index_bytes = calibration_index.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_adress, 0x40, &[0x10, index_bytes[0], index_bytes[1], index_bytes[2], index_bytes[3]]);
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.is_empty() {
            Err(TranslationError::NotEnoughData(1, data.len() as u8))?;
        }

        Ok(data[0] > 0)
    }

    pub fn get_calibration_id(&mut self, calibration_index: u32) -> Result<u32, DeviceError> { 
        let index_bytes = calibration_index.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_adress, 0x40, &[0x12, index_bytes[0], index_bytes[1], index_bytes[2], index_bytes[3]]);
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(1, data.len() as u8))?;
        }

        Ok(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn get_calibration_gas_unit(&mut self, calibration_index: u32) -> Result<GasUnit, DeviceError> { 
        let index_bytes = calibration_index.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_adress, 0x40, &[0x13, index_bytes[0], index_bytes[1], index_bytes[2], index_bytes[3]]);
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 3 {
            Err(TranslationError::NotEnoughData(3, data.len() as u8))?;
        }

        let prefix = Prefixes::from(i8::from_be_bytes([data[0]]));
        let unit = Units::from(data[1]);
        let time_base = TimeBases::from(data[2]);
        Ok(GasUnit {
            unit_prefex: prefix,
            medium_unit: unit,
            timebase: time_base,
        })
    }

    pub fn get_calibration_full_scale(&mut self, calibration_index: u32) -> Result<f32, DeviceError>{
        let index_bytes = calibration_index.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_adress, 0x40, &[0x14, index_bytes[0], index_bytes[1], index_bytes[2], index_bytes[3]]);
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn get_current_gas_id(&mut self) -> Result<u32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x44, &[0x12]);
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();
        
        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn get_current_gas_unit(&mut self) -> Result<GasUnit, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x44, &[0x13]);
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 3 {
            Err(TranslationError::NotEnoughData(3, data.len() as u8))?;
        }

        let prefix = Prefixes::from(i8::from_be_bytes([data[0]]));
        let unit = Units::from(data[1]);
        let time_base = TimeBases::from(data[2]);
        Ok(GasUnit {
            unit_prefex: prefix,
            medium_unit: unit,
            timebase: time_base,
        })
    }

    pub fn get_current_full_scale(&mut self) -> Result<f32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x44, &[0x14]);
        let _ = self.port.write(&frame.into_raw())?;
        let res = self.read_response()?;
        let data = res.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn get_calliration_id(&mut self) -> Result<u32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x45, &[]);
        let _ = self.port.write(&frame.into_raw())?;
        let res = self.read_response()?;
        let data = res.into_data();
        
        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    pub fn set_callibration(&mut self, calibration_index: u32) -> Result<(), DeviceError> {
        let cal_bytes = calibration_index.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_adress, 0x45, &cal_bytes);
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;

        Ok(())
    }

    pub fn get_baudrate(&mut self) -> Result<u32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x91, &[]);
        let _ = self.port.write(&frame.into_raw())?;

        let response = self.read_response()?;
        let data = response.into_data();
        
        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        let res = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);

        Ok(res)
    }

    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x91, &baudrate.to_be_bytes());
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;

        self.port.set_baud_rate(baudrate)?;

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

    pub fn read_response(&mut self) -> Result<MISOFrame, DeviceError> {
        let mut buff = [0_u8; 20];
        let mut out = ArrayVec::<u8, 518>::new();
        loop {
            let s = self.port.read(&mut buff)?;
            out.try_extend_from_slice(&buff[..s])?;
            if buff[s-1] == 0x7E && (s > 1 || out.len() > 1) {
                break;
            }
        }

        let frame = MISOFrame::from_bytes(&out);

        if !frame.is_ok() {
            Err(StateResponseError::from(frame.get_state()))?;
        }

        if !frame.validate_checksum() {
            Err(DeviceError::InvalidChecksum(frame.get_checksum(), frame.calculate_check_sum()))?;
        }

        Ok(frame)
    }
}

#[derive(Debug)]
pub enum DeviceError {
    IoError(std::io::Error),
    ShdlcError(TranslationError),
    StateResponse(StateResponseError),
    PortError(serialport::Error),
    InvalidChecksum(u8, u8),
    InvalidString,
}

impl Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => e.fmt(f),
            Self::ShdlcError(e) => e.fmt(f),
            Self::StateResponse(e) => e.fmt(f),
            Self::PortError(e) => e.fmt(f),
            Self::InvalidChecksum(recived, expected) => write!(f, "checksum recived: {:#02x} did not match expected value: {:#02x}", recived, expected),
            Self::InvalidString => write!(f, "invalid string data found"),
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

impl From<StateResponseError> for DeviceError {
    fn from(value: StateResponseError) -> Self {
        Self::StateResponse(value)
    }
}

impl From<serialport::Error> for DeviceError {
    fn from(value: serialport::Error) -> Self {
        Self::PortError(value)
    }
}

impl From<CapacityError> for DeviceError {
    fn from(_: CapacityError) -> Self {
        Self::ShdlcError(TranslationError::DataTooLarge)
    }
}

#[derive(Debug, PartialEq)]
pub enum StateResponseError {
    DataSizeError,
    UnknownCommand,
    ParameterError,
    I2CNackError,
    I2CMasterHoldError,
    CRCError,
    DataWriteError,
    MeasureLoopNotRunning,
    InvalidCalibration,
    SensorBusy,
    CommandNotAllowed,
    FatalError,
}

impl From<u8> for StateResponseError {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::DataSizeError,
            0x02 => Self::UnknownCommand,
            0x04 => Self::ParameterError,
            0x29 => Self::I2CNackError,
            0x2A => Self::I2CMasterHoldError,
            0x2B => Self::CRCError,
            0x2C => Self::DataWriteError,
            0x2D => Self::MeasureLoopNotRunning,
            0x33 => Self::InvalidCalibration,
            0x42 => Self::SensorBusy,
            0x32 => Self::CommandNotAllowed,
            0x7F => Self::FatalError,
            _ => Self::FatalError
        }
    }
}

impl Display for StateResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DataSizeError => write!(f, "illegal data size of MOSI frame or invalid frame"),
            Self::UnknownCommand => write!(f, "the device does not support or know this command"),
            Self::ParameterError => write!(f, "the sent parameter was out of range"),
            Self::I2CNackError => write!(f, "NACK recived from the I2C device"),
            Self::I2CMasterHoldError => write!(f, "master hold not released from I2C device"),
            Self::CRCError => write!(f, "checksum miss match occured"),
            Self::DataWriteError => write!(f, "sensor data read back differs from written value"),
            Self::MeasureLoopNotRunning => write!(f, "sensor mesaure loop not running or runs on wrong gas number"),
            Self::InvalidCalibration => write!(f, "no valid gas calibration at given index"),
            Self::SensorBusy => write!(f, "the sensor is busy at the moment, it takes 300ms to power-up after reset"),
            Self::CommandNotAllowed => write!(f, "command is not allowed in the current state"),
            Self::FatalError => write!(f, "an error without a specific code occured") 
            // wow fatal error very specifc shdlc 
        }
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use approx::assert_relative_eq;

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
            Err(DeviceError::StateResponse(StateResponseError::ParameterError)) => {},
            _ => panic!("expected, StateResponseError::ParameterError"),
        }
    }

    #[test]
    #[serial]
    fn set_get_set_setpoint() {
        let mut device = create_device();
        device.set_setpoint(2.0).unwrap();
        let res = device.get_setpoint().unwrap();
        device.set_setpoint(0.0).unwrap();
        assert_eq!(res, 2.0);
    }

    #[test]
    #[serial]
    fn reading_measured_values() {
        let mut device = create_device();
        let r1 = device.read_measured_value().unwrap();
        let r2 = device.read_average_measured_value(50).unwrap();
        println!("measured value: {}, average measured value: {}", r1, r2);
    }

    #[test]
    #[serial]
    fn read_wrong_measured_value() {
        let mut device = create_device();
        let r1 = device.read_average_measured_value(192);
        match r1 {
            Err(DeviceError::StateResponse(StateResponseError::ParameterError)) => {},
            _ => panic!("expected, StateReesponseError::ParameterError"),
        }
    }

    #[test]
    #[serial]
    fn get_current_full_scale() {
        let mut device = create_device();
        let r1 = device.get_current_full_scale().unwrap();
        println!("Current full scale {}", r1);
    }

    #[test]
    #[serial]
    fn set_setpoint_and_read_measured_value() {
        let mut device = create_device();
        let _ = device.set_setpoint_and_read_measured_value(1.5).unwrap();
        let r2 = device.get_setpoint().unwrap();
        device.set_setpoint(0.0).unwrap();

        assert_relative_eq!(1.5, r2);
    }

    #[test]
    #[serial]
    fn get_set_controller_gain() {
        let mut device = create_device();
        let original = device.get_controller_gain().unwrap();
        device.set_controller_gain(0.4).unwrap();
        let r2 = device.get_controller_gain().unwrap();
        device.set_controller_gain(original).unwrap();
        assert_relative_eq!(0.4, r2, epsilon = 0.0001);
    }

    #[test]
    #[serial]
    fn get_set_intial_step() {
        let mut device = create_device();
        let original = device.get_initial_step().unwrap();
        println!("intial step: {}", original);
        device.set_initial_step(0.4).unwrap();
        let r2 = device.get_initial_step().unwrap();
        device.set_initial_step(original).unwrap();
        assert_relative_eq!(0.4, r2);
    }

    #[test]
    #[serial]
    fn measure_raw_flow() {
        let mut device = create_device();
        let flow = device.measure_raw_flow().unwrap();
        println!("raw flow: {}", flow);
    }

    #[test]
    #[serial]
    fn measure_raw_thermal_conductivity() {
        let mut device = create_device();
        let conductivity = device.measure_raw_thermal_conductivity().unwrap();
        println!("raw thermal conductivity: {}", conductivity);
    }

    #[test]
    #[serial]
    fn measure_temperature() {
        let mut device = create_device();
        let temp = device.measure_temperature().unwrap();
        println!("Temperature in C: {}", temp);
    }

    #[test]
    #[serial]
    fn number_of_calibrations() {
        let mut device = create_device();
        let res = device.get_number_of_calibrations().unwrap();
        assert_eq!(res, 6);
    }
 
    #[test]
    #[serial]
    fn calibration_is_valid() {
        let mut device = create_device();
        let res = device.get_calibration_validity(0).unwrap();
        assert!(res);
    }

    #[test]
    #[serial]
    fn defualt_calibration() {
        let mut device = create_device();
        let unit = device.get_calibration_gas_unit(0).unwrap();
        let assert_unit = GasUnit {
           unit_prefex: Prefixes::Base,
           timebase: TimeBases::Minute,
           medium_unit: Units::StandardLiter,
        };
        assert_eq!(unit, assert_unit);
    }

    #[test]
    #[serial]
    fn gas_calibration_functions() {
        let mut device = create_device();
        let unit = device.get_calibration_gas_unit(0).unwrap();
        let fs = device.get_calibration_full_scale(0).unwrap();
        let id = device.get_current_gas_id().unwrap();
        println!("fs: {}", fs);
        println!("unit: {:?}", unit);
        println!("id: {}", id);
    }
    
    #[test]
    #[serial]
    fn set_and_reset_calibration() {
        let mut device = create_device();
        let original = device.get_calliration_id().unwrap();
        device.set_callibration(1).unwrap();
        assert_eq!(1, device.get_calliration_id().unwrap());
        device.set_callibration(original).unwrap();
    }
}
