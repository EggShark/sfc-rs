//! The SFC6xxx device and associated functions

use std::ffi::CString;
use std::fmt::Display;

use arrayvec::{ArrayVec, CapacityError};
use serialport::SerialPort;

use crate::gasunit::{GasUnit, Prefixes, TimeBases, Units};
use crate::shdlc::{MISOFrame, MOSIFrame, TranslationError, Version};

/// A representation of a physical SFC6XXX. It must be given a valid serial port
/// in order to operate.
#[derive(Debug)]
pub struct Device<T: SerialPort> {
    port: T,
    slave_adress: u8,
}

impl<T: SerialPort> Device<T> {
    /// The device can be created by passing a serial port and slave adress like so:
    /// ```no_run
    /// use sfc6xxx_rs::device::Device;
    /// let test_port = serialport::new("ttyUSB0", 115200).open_native().unwrap();
    /// let device = Device::new(test_port, 0).unwrap();
    /// ```
    pub fn new(mut serial_port: T, slave_adress: u8) -> Result<Self, DeviceError> {
        serial_port.set_timeout(std::time::Duration::from_millis(600))?;

        let mut device = Self {
            port: serial_port,
            slave_adress,
        };

        // simple command ot check if its a valid SHDLC device
        let _ = device.get_baudrate()?;

        Ok(device)
    }

    /// Returns the current flow setpoint as a physical value in SLM
    pub fn get_setpoint(&mut self) -> Result<f32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x00, &[0x01])?;
        let _ = self.port.write(&frame.into_raw())?;
        let res = self.read_response()?;
        let data = res.into_data();
        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    /// Sets the flow setpoint as a physical value. The range of valid set points is 0.0 to
    /// [Device::get_current_full_scale]. The setpoint will be set to 0 if the calibration is ever
    /// changed.
    pub fn set_setpoint(&mut self, setpoint: f32) -> Result<(), DeviceError> {
        let setpoint_bytes = setpoint.to_be_bytes();
        let frame = MOSIFrame::new(
            self.slave_adress,
            0x00,
            &[
                0x01,
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

    /// Returns the latest measured flow as physical value
    pub fn read_measured_value(&mut self) -> Result<f32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x08, &[0x01])?;
        let _ = self.port.write(&frame.into_raw())?;
        let res = self.read_response()?;
        let data = res.into_data();
        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    /// Returns the average of given numbers of flow measurment as a physical value. Each
    /// measurment takes 1ms so the command response time depends on the number of measurements.
    /// Addtionaly the number of measurments must be between 0 and 100 other wise it will return a
    /// [StateResponseError::ParameterError].
    pub fn read_average_measured_value(
        &mut self,
        measurment_count: u8,
    ) -> Result<f32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x08, &[0x11, measurment_count])?;
        let raw = frame.into_raw();

        let _ = self.port.write(&raw)?;
        let res = self.read_response()?;
        let data = res.into_data();
        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    /// Sets the set point and reads the measured value in one SHDLC command
    pub fn set_setpoint_and_read_measured_value(
        &mut self,
        setpoint: f32,
    ) -> Result<f32, DeviceError> {
        let setpoint_bytes = setpoint.to_be_bytes();
        let frame = MOSIFrame::new(
            self.slave_adress,
            0x03,
            &[
                0x01,
                setpoint_bytes[0],
                setpoint_bytes[1],
                setpoint_bytes[2],
                setpoint_bytes[3],
            ],
        )?;
        let _ = self.port.write(&frame.into_raw())?;
        let res = self.read_response()?;
        let data = res.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    /// Returns the controller gain
    pub fn get_controller_gain(&mut self) -> Result<f32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x22, &[0x00])?;
        let _ = self.port.write(&frame.into_raw())?;
        let res = self.read_response()?;
        let data = res.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }
        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    /// Sets the controller gain to the desired value
    pub fn set_controller_gain(&mut self, gain: f32) -> Result<(), DeviceError> {
        let gain_bytes = gain.to_be_bytes();
        let frame = MOSIFrame::new(
            self.slave_adress,
            0x22,
            &[
                0x00,
                gain_bytes[0],
                gain_bytes[1],
                gain_bytes[2],
                gain_bytes[3],
            ],
        )?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    /// Gets the device intital step
    pub fn get_initial_step(&mut self) -> Result<f32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x22, &[0x03])?;
        let _ = self.port.write(&frame.into_raw())?;
        let res = self.read_response()?;
        let data = res.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }
        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    /// Sets the initial step. This is stored in non-volatile memory and will be cleared
    /// after a device reset.
    pub fn set_initial_step(&mut self, step: f32) -> Result<(), DeviceError> {
        let step_bytes = step.to_be_bytes();
        let frame = MOSIFrame::new(
            self.slave_adress,
            0x22,
            &[
                0x03,
                step_bytes[0],
                step_bytes[1],
                step_bytes[2],
                step_bytes[3],
            ],
        )?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    /// Retunrs the measured flow in raw ticks
    pub fn measure_raw_flow(&mut self) -> Result<u16, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x30, &[0x00])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 2 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(u16::from_be_bytes([data[0], data[1]]))
    }

    /// Preforms a thermal conductivity measurement and returns the measure raw tick value.
    /// The valve is automatically closed during the measurment
    pub fn measure_raw_thermal_conductivity(&mut self) -> Result<u16, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x30, &[0x02])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 2 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(u16::from_be_bytes([data[0], data[1]]))
    }

    /// Measures the temperature of the flow sensor in degrees celcius
    pub fn measure_temperature(&mut self) -> Result<f32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x30, &[0x10])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    /// Gets the number of calibrations that the device memory is able to hold.
    /// Not all calibrations actually contain a valid calibration. Use [Device::get_calibration_validity]
    /// to see which calibrations are valid and can be used
    pub fn get_number_of_calibrations(&mut self) -> Result<u32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x40, &[0x00])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }
        Ok(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    /// Checks if a calibration at the specific index is valid
    pub fn get_calibration_validity(
        &mut self,
        calibration_index: u32,
    ) -> Result<bool, DeviceError> {
        let index_bytes = calibration_index.to_be_bytes();
        let frame = MOSIFrame::new(
            self.slave_adress,
            0x40,
            &[
                0x10,
                index_bytes[0],
                index_bytes[1],
                index_bytes[2],
                index_bytes[3],
            ],
        )?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.is_empty() {
            Err(TranslationError::NotEnoughData(1, data.len() as u8))?;
        }

        Ok(data[0] > 0)
    }

    /// Gets the gas ID of the specifc calibration index.
    pub fn get_calibration_gas_id(&mut self, calibration_index: u32) -> Result<u32, DeviceError> {
        let index_bytes = calibration_index.to_be_bytes();
        let frame = MOSIFrame::new(
            self.slave_adress,
            0x40,
            &[
                0x12,
                index_bytes[0],
                index_bytes[1],
                index_bytes[2],
                index_bytes[3],
            ],
        )?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(1, data.len() as u8))?;
        }

        Ok(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    /// Gets the gas unit of a specifc calibration index see [GasUnit] for more information.
    pub fn get_calibration_gas_unit(
        &mut self,
        calibration_index: u32,
    ) -> Result<GasUnit, DeviceError> {
        let index_bytes = calibration_index.to_be_bytes();
        let frame = MOSIFrame::new(
            self.slave_adress,
            0x40,
            &[
                0x13,
                index_bytes[0],
                index_bytes[1],
                index_bytes[2],
                index_bytes[3],
            ],
        )?;
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

    /// Returns the full scale flow of a specifc calibration index.
    pub fn get_calibration_full_scale(
        &mut self,
        calibration_index: u32,
    ) -> Result<f32, DeviceError> {
        let index_bytes = calibration_index.to_be_bytes();
        let frame = MOSIFrame::new(
            self.slave_adress,
            0x40,
            &[
                0x14,
                index_bytes[0],
                index_bytes[1],
                index_bytes[2],
                index_bytes[3],
            ],
        )?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    /// Gets the gas ID of the currently active calibration
    pub fn get_current_gas_id(&mut self) -> Result<u32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x44, &[0x12])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    /// Gets the gas unit of the currently active calibration. See [GasUnit] for more
    /// information
    pub fn get_current_gas_unit(&mut self) -> Result<GasUnit, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x44, &[0x13])?;
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

    /// Gets the full scale flow of the currently active calibration.
    pub fn get_current_full_scale(&mut self) -> Result<f32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x44, &[0x14])?;
        let _ = self.port.write(&frame.into_raw())?;
        let res = self.read_response()?;
        let data = res.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(f32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    /// Gets the calibration index of the currently active calibration.
    pub fn get_calliration_number(&mut self) -> Result<u32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x45, &[])?;
        let _ = self.port.write(&frame.into_raw())?;
        let res = self.read_response()?;
        let data = res.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        Ok(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    /// Changes the calibration to the new calibration at the specified index. This command
    /// stops the controller by closing the valve. Additonly this is stored in presitent memory and
    /// will remain after a device reset.
    pub fn set_callibration(&mut self, calibration_index: u32) -> Result<(), DeviceError> {
        let cal_bytes = calibration_index.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_adress, 0x45, &cal_bytes)?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;

        Ok(())
    }

    /// Changes the calibration to the new calibration at the specified index. This command stops
    /// the controller by closing the valve. This will be started in volatile memory and will not
    /// presit after a device reset.
    pub fn set_callibration_volitile(&mut self, calibration_index: u32) -> Result<(), DeviceError> {
        let cal_bytes = calibration_index.to_be_bytes();
        let frame = MOSIFrame::new(self.slave_adress, 0x46, &cal_bytes)?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;
        Ok(())
    }

    /// Returns the slave adress of the SHDLC device
    pub fn get_slave_adress(&mut self) -> Result<u8, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x90, &[])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.is_empty() {
            Err(TranslationError::NotEnoughData(1, 0))?;
        }

        Ok(data[0])
    }

    /// Sets slave adress of the SHDLC device. The slave adress is stored in non-volatile memory
    /// and therefore will presist after a device reset. Next time the device is connected be sure
    /// to use the new address. Aditionally make sure there is only one device with this address on
    /// the bus. Otherwise there will be communication errors that can only be fixed by
    /// disconnecting one of the devices.
    pub fn set_slave_adress(&mut self, new_adress: u8) -> Result<(), DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x90, &[new_adress])?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;

        self.slave_adress = new_adress;
        Ok(())
    }

    /// Gets the baudrate of the SHDLC device.
    pub fn get_baudrate(&mut self) -> Result<u32, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x91, &[])?;
        let _ = self.port.write(&frame.into_raw())?;

        let response = self.read_response()?;
        let data = response.into_data();

        if data.len() < 4 {
            Err(TranslationError::NotEnoughData(4, data.len() as u8))?;
        }

        let res = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);

        Ok(res)
    }

    /// Sets the buadrate of the device. The buadrate is stored in non-volatile memory
    /// and will presist after a device reset. The next time you connect to the device make
    /// sure to use the new baudrate. Allowed buadrate values are `19200`, `38400`, `57600`,
    /// and `115200`.
    pub fn set_baudrate(&mut self, baudrate: u32) -> Result<(), DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0x91, &baudrate.to_be_bytes())?;
        let _ = self.port.write(&frame.into_raw())?;
        let _ = self.read_response()?;

        self.port.set_baud_rate(baudrate)?;

        Ok(())
    }

    /// Gets the product type from the device
    pub fn get_product_type(&mut self) -> Result<String, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0xD0, &[0x00])?;
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

    /// Gets the product name from the device
    pub fn get_product_name(&mut self) -> Result<String, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0xD0, &[0x01])?;
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

    /// Gets the article code of the device. This information is also contained on the
    /// product label.
    pub fn get_article_code(&mut self) -> Result<String, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0xD0, &[0x02])?;
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

    /// Gets the serial number of the SFC6xxx sensor as a hex String matching the 
    /// serial number printed on the device.
    pub fn get_serial_number(&mut self) -> Result<String, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0xD0, &[0x03])?;
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

    /// Gets the version information for the hardware, firmware, and SHDLC protocol.
    pub fn get_version(&mut self) -> Result<Version, DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0xD1, &[])?;
        let _ = self.port.write(&frame.into_raw())?;
        let data = self.read_response()?.into_data();

        if data.len() < 7 {
            Err(DeviceError::ShdlcError(TranslationError::NotEnoughData(
                7,
                data.len() as u8,
            )))?;
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

    /// Resets the device which has the same effect as a power cycle. Please allow 300ms for the
    /// device to power on
    pub fn reset_device(&mut self) -> Result<(), DeviceError> {
        let frame = MOSIFrame::new(self.slave_adress, 0xD3, &[])?;
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

/// Errors the device can encounter while operating
#[derive(Debug)]
pub enum DeviceError {
    /// An error when writing data or reading data from the device.
    IoError(std::io::Error),
    ShdlcError(TranslationError),
    StateResponse(StateResponseError),
    PortError(serialport::Error),
    /// The checksum recived was the first value when it expected the second
    InvalidChecksum(u8, u8),
    /// An invalid string was sent from the device. Either missing the null terminator byte
    /// or was not valid ASCII.
    InvalidString,
}

impl Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => e.fmt(f),
            Self::ShdlcError(e) => e.fmt(f),
            Self::StateResponse(e) => e.fmt(f),
            Self::PortError(e) => e.fmt(f),
            Self::InvalidChecksum(recived, expected) => write!(
                f,
                "checksum recived: {:#02x} did not match expected value: {:#02x}",
                recived, expected
            ),
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

/// Errors sent back from a MISO frame.
#[derive(Debug, PartialEq)]
pub enum StateResponseError {
    /// Illegal data size of the MOSI frame. Either an invalid frame was sent or
    /// the firmware does not support the requested feature
    DataSizeError,
    /// The device does not know this command.
    UnknownCommand,
    /// A sent parameter is out of range.
    ParameterError,
    /// NACK recived from the I2C device.
    I2CNackError,
    /// Master hold not released in I2C.
    I2CMasterHoldError,
    /// I2C CRC missmatch
    CRCError,
    /// Sensor data read back differs from written value
    DataWriteError,
    /// Sensor measure loop is not running or runs on wrong gas number.
    MeasureLoopNotRunning,
    /// No valid gas calibration at given index
    InvalidCalibration,
    /// The sensor is busy at the moment
    SensorBusy,
    /// Command is not allwed in the current state.
    CommandNotAllowed,
    /// An error without a specifc error code.
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
            _ => Self::FatalError,
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
            Self::MeasureLoopNotRunning => write!(
                f,
                "sensor mesaure loop not running or runs on wrong gas number"
            ),
            Self::InvalidCalibration => write!(f, "no valid gas calibration at given index"),
            Self::SensorBusy => write!(
                f,
                "the sensor is busy at the moment, it takes 300ms to power-up after reset"
            ),
            Self::CommandNotAllowed => write!(f, "command is not allowed in the current state"),
            Self::FatalError => write!(f, "an error without a specific code occured"), // wow fatal error very specifc shdlc
        }
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use serial_test::serial;

    #[cfg(target_os = "windows")]
    use serialport::COMPort;
    #[cfg(target_os = "linux")]
    use serialport::TTYPort;

    #[cfg(target_os = "linux")]
    const PORT: &str = "/dev/ttyUSB0";
    #[cfg(target_os = "windows")]
    const PORT: &str = "COM4";

    use super::*;

    #[cfg(target_os = "linux")]
    type SP = TTYPort;
    #[cfg(target_os = "windows")]
    type SP = COMPort;

    fn create_device() -> Device<SP> {
        let test_port = serialport::new(PORT, 115200).open_native().unwrap();
        Device::new(test_port, 0).unwrap()
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
            Err(DeviceError::StateResponse(StateResponseError::ParameterError)) => {}
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
            Err(DeviceError::StateResponse(StateResponseError::ParameterError)) => {}
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

    // ignored due to the limited write cycles of the flash memory
    #[test]
    #[serial]
    #[ignore]
    fn set_and_reset_calibration() {
        let mut device = create_device();
        let original = device.get_calliration_number().unwrap();
        device.set_callibration(1).unwrap();
        assert_eq!(1, device.get_calliration_number().unwrap());
        device.set_callibration(original).unwrap();
    }

    #[test]
    #[serial]
    fn set_callibration_volitile_and_reset() {
        let mut device = create_device();
        device.set_callibration_volitile(2).unwrap();
        device.reset_device().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(400));
        assert_eq!(1, device.get_calliration_number().unwrap());
    }

    #[test]
    #[serial]
    fn set_slave_adress_and_back() {
        let mut device = create_device();
        let original = device.get_slave_adress().unwrap();
        device.set_slave_adress(2).unwrap();
        assert_eq!(2, device.get_slave_adress().unwrap());
        device.set_slave_adress(original).unwrap();
    }

    #[test]
    #[serial]
    fn get_firmware_version() {
        let mut device = create_device();
        let v = device.get_version().unwrap();
        println!("{:?}", v);
    }
}
