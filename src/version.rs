#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Version {
    pub firmware_major: u8,
    pub firmware_minor: u8,
    pub debug: bool,
    pub hardware_major: u8,
    pub hardware_minor: u8,
    pub protocol_major: u8,
    pub protocol_minor: u8,
}
