#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputSourceConfig {
    Controller,
    ForceClosed,
    ForceOpen,
    Hold,
    UserDefined(f32),
}

impl Into<u8> for InputSourceConfig {
    fn into(self) -> u8 {
        match self {
            Self::Controller => 0x01,
            Self::ForceClosed => 0x02,
            Self::ForceOpen => 0x03,
            Self::Hold => 0x04,
            Self::UserDefined(_) => 0x10,
        }
    }
}
