use std::fmt::Display;

use arrayvec::ArrayVec;

pub const START_STOP: u8 = 0x7E;
pub const START_SWAP: u8 = 0x5E;
pub const ESCAPE: u8 = 0x7D;
pub const ESCAPE_SWAP: u8 = 0x5D;
pub const XON: u8 = 0x11;
pub const XON_SWAP: u8 = 0x31;
pub const XOFF: u8 = 0x13;
pub const XOFF_SWAP: u8 = 0x33;

pub struct MOSIFrame {
    address: u8,
    command: u8,
    data_length: u8,
    raw: ArrayVec<u8, 518>,
    checksum: u8,
}

impl MOSIFrame {
    pub fn new(address: u8, command: u8, data: &[u8]) -> Self {
        let mut pre_procressed: ArrayVec<u8, 258> = ArrayVec::new();
        pre_procressed.push(address);
        pre_procressed.push(command);
        pre_procressed.push(data.len() as u8);
        pre_procressed.try_extend_from_slice(data).unwrap();

        let data_length = data.len() as u8;
        let raw = to_shdlc(&pre_procressed).unwrap();
        Self {
            address,
            command,
            data_length,
            raw,
            checksum: 0,
        }
    }

    pub fn check_sum(&self) -> u8 {
        self.checksum
    }

    pub fn into_raw(self) -> ArrayVec<u8, 518> {
        self.raw
    }
}

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

    pub fn is_ok(&self) -> bool {
        self.state == 0
    }

    pub fn get_state(&self) -> u8 {
        self.state
    }

    pub fn get_checksum(&self) -> u8 {
        self.checksum
    }

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

    pub fn validate_checksum(&self) -> bool {
       self.calculate_check_sum() == self.checksum
    }

    pub fn into_data(self) -> ArrayVec<u8, 255> {
        self.data
    }
}

pub fn calculate_check_sum(data: &[u8]) -> u8 {
    data.iter().fold(0, |acc: u8, x| acc.wrapping_add(*x)) ^ 0xFF_u8
}


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

pub fn from_shdlc(data: &[u8]) -> Result<ArrayVec<u8, 262>, TranslationError> {
    let mut out = ArrayVec::new();

    let mut iter = data[1..data.len()-1].iter();
    
    while let Some(&byte) = iter.next() {
        match byte {
            ESCAPE => match iter.next() {
                Some(0x5E) => out.push(START_STOP),
                Some(0x5D) => out.push(ESCAPE),
                Some(0x31) => out.push(XON),
                Some(0x33) => out.push(XOFF),
                Some(b) => Err(TranslationError::MissingEscapedData(*b))?,
                None => Err(TranslationError::MissingEscapedData(0))?,
            }
            START_STOP => Err(TranslationError::FrameEndInData)?,
            _ => out.push(byte),
        }
    }

    Ok(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationError {
    DataTooLarge,
    NotEnoughData(u8, u8),
    MissingEscapedData(u8),
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


#[cfg(test)]
mod tests {
    use super::calculate_check_sum;

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
}
