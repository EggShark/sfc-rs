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
    data: [u8; 255],
    checksum: u8,
}

pub struct MISOFrame {
    address: u8,
    command: u8,
    data_length: u8,
    data: [u8; 255],
    checksum: u8,
}

pub fn to_shdlc(data: &[u8]) -> Result<ArrayVec<u8, 256>, TranslationError> {
    let mut out = ArrayVec::new();
    if data.len() > 256 {
        Err(TranslationError::DataTooLarge)?;
    }

    for &b in data {
        if out.len() == 255 {
            Err(TranslationError::DataTooLarge)?;
        }

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

    Ok(out)
}

pub fn from_shdlc(data: &[u8]) -> Result<ArrayVec<u8, 256>, TranslationError> {
    let mut out = ArrayVec::new();

    let mut iter = data.iter();
    
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

pub enum TranslationError {
    DataTooLarge,
    MissingEscapedData,
    FrameEndInData,
}
