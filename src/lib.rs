use arrayvec::ArrayVec;

pub const START_STOP: u8 = 0x7E;
pub const ESCAPE: u8 = 0x7D;
pub const XON: u8 = 0x11;
pub const XOFF: u8 = 0x13;

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
                out.push(0x7D);
                out.push(0x5E);
            }
            ESCAPE => {
                out.push(0x7D);
                out.push(0x5D);
            }
            XON => {
                out.push(0x7D);
                out.push(0x31);
            }
            XOFF => {
                out.push(0x7D);
                out.push(0x33);
            }
            _ => out.push( b)
        }
    }

    Ok(out)
}

fn from_shdlc() {
    
}

pub enum TranslationError {
    DataTooLarge
}