use crate::shdlc::TranslationError;

use arrayvec::CapacityError;

use std::fmt::Display;

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
