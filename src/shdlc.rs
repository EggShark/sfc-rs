//! Functions and structs relating to the underlying SHDLC protocol deffintions of these types can
//! be seen [here](https://sensirion.com/media/documents/88CA2961/65156AEC/GF_AN_SFX6000_SHDLCGuide1.1.pdf)

use std::fmt::Display;

use arrayvec::{ArrayVec, CapacityError};

pub const START_STOP: u8 = 0x7E;
pub const START_SWAP: u8 = 0x5E;
pub const ESCAPE: u8 = 0x7D;
pub const ESCAPE_SWAP: u8 = 0x5D;
pub const XON: u8 = 0x11;
pub const XON_SWAP: u8 = 0x31;
pub const XOFF: u8 = 0x13;
pub const XOFF_SWAP: u8 = 0x33;

/// A representation of a SHDLC Master Out Slave In frame.
/// Each frame contains a 1 byte Frame start/end symbol. The slave adress of the device.
/// The command byte. The length of the data being 0-255. The actuall data a checksum followed 
/// by the Frame end byte.
pub struct MOSIFrame {
    address: u8,
    command: u8,
    data_length: u8,
    raw: ArrayVec<u8, 518>,
    checksum: u8,
}

impl MOSIFrame {
    /// Constructs a MOSI frame from the adress, command, and data. This will automatically
    /// translate the data using SHDLC byte stuffing.
    pub fn new(address: u8, command: u8, data: &[u8]) -> Result<Self, TranslationError> {
        let mut pre_procressed: ArrayVec<u8, 258> = ArrayVec::new();
        pre_procressed.push(address);
        pre_procressed.push(command);
        pre_procressed.push(data.len() as u8);
        pre_procressed.try_extend_from_slice(data)?;

        let data_length = data.len() as u8;
        let raw = to_shdlc(&pre_procressed)?;
        Ok(Self {
            address,
            command,
            data_length,
            raw,
            checksum: 0,
        })
    }

    /// Returns the slave adress of the command
    pub fn get_address(&self) -> u8 {
        self.address
    }

    /// Returns the command number/byte of 
    pub fn get_command_number(&self) -> u8 {
        self.command
    }

    /// Returns the length of the data pre byte stuffing
    pub fn get_data_length(&self) -> u8 {
        self.data_length
    }

    /// Returns the checksum of the MOSIframe
    pub fn check_sum(&self) -> u8 {
        self.checksum
    }

    /// returns the underlying ArrayVec ready to be written to the device
    pub fn into_raw(self) -> ArrayVec<u8, 518> {
        self.raw
    }

    /// Validates the checksum and returns true if its valid
    pub fn validate_checksum(&self) -> bool {
        let raw = from_shdlc(&self.raw).unwrap();
        let ck = calculate_check_sum(&raw[1..raw.len()-2]);
        ck == self.checksum
    }
}

/// The Master In Slave Out frame or the response from the device. Simillar
/// to the MOSI frame it begins with a start byte. The slave adress of the 
/// responding device. The command number byte. The State byte. The data length.
/// Followed by the data, the checksum and a stop byte.
#[derive(Debug)]
pub struct MISOFrame {
    address: u8,
    command: u8,
    data_length: u8,
    state: u8,
    data: ArrayVec<u8, 255>,
    checksum: u8,
}


impl MISOFrame {
    /// parses the data from raw bytes should come from a bytestream of the device
    pub fn from_bytes(data: &[u8]) -> Self {
        let decoded = from_shdlc(data).unwrap();
        let address = decoded[0];
        let command = decoded[1];
        let state = decoded[2];
        let data_length = decoded[3];
        let checksum = decoded[decoded.len() - 1];
        let mut data = ArrayVec::new();
        let _ = data.try_extend_from_slice(&decoded[4..4+data_length as usize]);

        Self {
            address,
            command,
            data_length,
            state,
            data,
            checksum
        }
    }

    /// Reads the state byte and returns true if its 0
    pub fn is_ok(&self) -> bool {
        self.state == 0
    }

    /// Returns the state byte of the MOSI frame
    pub fn get_state(&self) -> u8 {
        self.state
    }

    /// Returns the checksum
    pub fn get_checksum(&self) -> u8 {
        self.checksum
    }

    /// Calculates the checksum of the MOSI frame
    pub fn calculate_check_sum(&self) -> u8 {
        let mut ck: u8 = 0;
        ck = ck.wrapping_add(self.address);
        ck = ck.wrapping_add(self.command);
        ck = ck.wrapping_add(self.data_length);
        ck = ck.wrapping_add(self.state);
        ck = ck.wrapping_add(self.data.iter().fold(0, |acc, x| acc.wrapping_add(*x)));
        ck ^= 0xFF;
        ck
    }

    /// validates the checksum from the device
    pub fn validate_checksum(&self) -> bool {
       self.calculate_check_sum() == self.checksum
    }

    /// Turns the frame directly into the underyling data pre byte stuffing
    pub fn into_data(self) -> ArrayVec<u8, 255> {
        self.data
    }
}

/// Cacluates the SHDLC checksum from a byte array
pub fn calculate_check_sum(data: &[u8]) -> u8 {
    data.iter().fold(0, |acc: u8, x| acc.wrapping_add(*x)) ^ 0xFF_u8
}

/// Does the byte stuffing operations in order to send data to the physical device
pub fn to_shdlc(data: &[u8]) -> Result<ArrayVec<u8, 518>, TranslationError> {
    let mut out = ArrayVec::new();

    out.push(START_STOP);
    let ck = calculate_check_sum(data);
    

    if data.len() > 258 {
        Err(TranslationError::DataTooLarge)?;
    }

    for &b in data {
        match b {
            START_STOP => {
                out.push(ESCAPE);
                out.push(START_SWAP);
            }
            ESCAPE => {
                out.push(ESCAPE);
                out.push(ESCAPE_SWAP);
            }
            XON => {
                out.push(ESCAPE);
                out.push(XON_SWAP);
            }
            XOFF => {
                out.push(ESCAPE);
                out.push(XOFF_SWAP);
            }
            _ => out.push(b)
        }
    }
    out.push(ck);

    out.push(START_STOP);

    Ok(out)
}

/// Translates the byte data from the device into standard data without bytestuffing
pub fn from_shdlc(data: &[u8]) -> Result<ArrayVec<u8, 262>, TranslationError> {
    let mut out = ArrayVec::new();

    let mut iter = data[1..data.len()-1].iter();
    
    while let Some(&byte) = iter.next() {
        match byte {
            ESCAPE => match iter.next() {
                Some(0x5E) => out.try_push(START_STOP)?,
                Some(0x5D) => out.try_push(ESCAPE)?,
                Some(0x31) => out.try_push(XON)?,
                Some(0x33) => out.try_push(XOFF)?,
                Some(b) => Err(TranslationError::MissingEscapedData(*b))?,
                None => Err(TranslationError::MissingEscapedData(0))?,
            }
            START_STOP => Err(TranslationError::FrameEndInData)?,
            _ => out.try_push(byte)?,
        }
    }

    Ok(out)
}

/// Each type of error that can occur from translating to and from SHDLC
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationError {
    /// Too much data was supplied. Data frame was larger than 255 bytes long
    DataTooLarge,
    /// There was not enough data. First value is expected number second value is found number
    NotEnoughData(u8, u8),
    /// The escape byte 0x7D was encountered but a valid swap character was not found 
    MissingEscapedData(u8),
    /// The frame end byte was fround inside the data
    FrameEndInData,
}

impl Display for TranslationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DataTooLarge => write!(f, "data Exceeded maxium length of 256"),
            Self::FrameEndInData => write!(f, "the frame end byte ({:#02x}) was found inside the data", START_STOP),
            Self::NotEnoughData(expected, found) => write!(f, "was epxected at least {} bytes, found {} bytes", expected, found),
            Self::MissingEscapedData(b) => write!(f, "the escape byte ({:#02x}) was placed before an invalid escaped byte: ({:#02x})", ESCAPE, b),
        }
    }
}

impl<T> From<CapacityError<T>> for TranslationError {
    fn from(_: CapacityError<T>) -> Self {
        Self::DataTooLarge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_guide() {
        let data = [0, 0x02, 0x43, 0x04, 0x64, 0xA0, 0x22, 0xFC];
        let ck = calculate_check_sum(&data);
        assert_eq!(ck, 0x94);
    }

    #[test]
    fn one_two_three() {
        let data = [0, 1, 2, 3, 4, 5, 6];
        let ck = calculate_check_sum(&data);
        assert_eq!(ck, 234);
    }

    #[test]
    fn check_sum_over_flow() {
        let data = [0, 200, 201, 202];
        let ck = calculate_check_sum(&data);
        assert_eq!(ck, 164);
    }

    #[test]
    fn too_much_data_in() {
        let vec = vec![0_u8; 1000];
        let attempt = to_shdlc(&vec);
        assert_eq!(attempt, Err(TranslationError::DataTooLarge));
    }

    #[test]
    fn too_much_data_out() {
        let vec = vec![0_u8; 1000];
        let attempt = from_shdlc(&vec);
        assert_eq!(attempt, Err(TranslationError::DataTooLarge));
    }
}
