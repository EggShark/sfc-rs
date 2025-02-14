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
        let data_length = data.len() as u8;
        let raw = to_shdlc(address, command, data_length, data).unwrap();
        Self {
            address,
            command,
            data_length,
            raw,
            checksum: 0,
        }
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

    pub fn get_state(&self) -> u8 {
        self.state
    }

    pub fn validate_check_sum(&self) -> bool {
        let mut ck: u8 = 0;
        ck = ck.wrapping_add(self.address);
        ck = ck.wrapping_add(self.command);
        ck = ck.wrapping_add(self.data_length);
        ck = ck.wrapping_add(self.state);
        ck = ck.wrapping_add(self.data.iter().fold(0, |acc, x| acc + acc.wrapping_add(*x)));
        ck ^= 0xFF;
        ck == self.checksum
    }

    pub fn into_data(self) -> ArrayVec<u8, 255> {
        self.data
    }
}

pub fn calculate_check_sum(frame: &[u8]) -> u8 {
    frame[1..].iter().fold(0_u8, |acc, x| acc.wrapping_add(*x)) ^ 0xFF_u8
}

pub fn to_shdlc(address: u8, command: u8, data_len: u8, data: &[u8]) -> Result<ArrayVec<u8, 518>, TranslationError> {
    let mut out = ArrayVec::new();
    out.push(START_STOP);
    out.push(address);
    out.push(command);
    
    match data_len {
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
        _ => out.push(data_len),
    }

    if data.len() > 255 {
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
    let checksum = calculate_check_sum(&out);
    out.push(checksum);

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
                _ => Err(TranslationError::MissingEscapedData)?,
            }
            START_STOP => Err(TranslationError::FrameEndInData)?,
            _ => out.push(byte),
        }
    }

    Ok(out)
}

#[derive(Debug)]
pub enum TranslationError {
    DataTooLarge,
    MissingEscapedData,
    FrameEndInData,
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
