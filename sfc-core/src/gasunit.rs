//! Gas calibrations come with units of measurments. The format is a (SI prefix * Flow unit)/Time
//! Unit

use std::fmt::Display;

/// GasUnit contains a base unit its SI prefix and the time base such as: centimeter per
/// minute. Often used when checking current calibration settings of a device. 
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GasUnit {
    pub unit_prefex: Prefixes,
    pub medium_unit: Units,
    pub timebase: TimeBases,
}

impl GasUnit {
    pub fn from_be_bytes(bytes: [u8; 3]) -> Self {
        Self {
            unit_prefex: i8::from_be_bytes([bytes[0]]).into(),
            medium_unit: bytes[1].into(),
            timebase: bytes[2].into()
        }
    }
}

/// SI prefixes that the device can transmit
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Prefixes {
    Yocto, // -24
    Zepto, // -21
    Atto,  // -18
    Femto, // -15
    Pico,  // -12
    Nano,  // -9
    Micro, // -6
    Milli, // -3
    Centi, // -2
    Deci,  // -1
    Base,  //  0
    Deca,  //  1
    Hecto, //  2
    Kilo,  //  3
    Mega,  //  6
    Giga,  //  9
    Tera,  //  12
    Peta,  //  15
    Exa,   //  18
    Zetta, //  21
    Yotta, //  24
    Undefined,
}

impl From<i8> for Prefixes {
    fn from(value: i8) -> Self {
        match value {
            -24 => Self::Yocto,
            -21 => Self::Zepto,
            -18 => Self::Atto,
            -15 => Self::Femto,
            -12 => Self::Pico,
            -9 => Self::Nano,
            -6 => Self::Micro,
            -3 => Self::Milli,
            -2 => Self::Centi,
            -1 => Self::Deci,
            0 => Self::Base,
            1 => Self::Deca,
            2 => Self::Hecto,
            3 => Self::Kilo,
            6 => Self::Mega,
            9 => Self::Giga,
            12 => Self::Tera,
            15 => Self::Peta,
            18 => Self::Exa,
            21 => Self::Zetta,
            24 => Self::Yotta,
            _ => Self::Undefined,
        }
    }
}

impl From<Prefixes> for i8 {
    fn from(value: Prefixes) -> Self {
        match value {
            Prefixes::Yocto => -24,
            Prefixes::Zepto => -21,
            Prefixes::Atto => -18,
            Prefixes::Femto => -15,
            Prefixes::Pico => -12,
            Prefixes::Nano => -9,
            Prefixes::Micro => -6,
            Prefixes::Milli => -3,
            Prefixes::Centi => -2,
            Prefixes::Deci => -1,
            Prefixes::Base => 0,
            Prefixes::Deca => 1,
            Prefixes::Hecto => 2,
            Prefixes::Kilo => 3,
            Prefixes::Mega => 6,
            Prefixes::Giga => 9,
            Prefixes::Tera => 12,
            Prefixes::Peta => 15,
            Prefixes::Exa => 18,
            Prefixes::Zetta => 21,
            Prefixes::Yotta => 24,
            Prefixes::Undefined => 0x7F,
        }
    }
}

impl Display for Prefixes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Yocto => write!(f, "y"),
            Self::Zepto => write!(f, "z"),
            Self::Atto => write!(f, "a"),
            Self::Femto => write!(f, "f"),
            Self::Pico => write!(f, "p"),
            Self::Nano => write!(f, "n"),
            Self::Micro => write!(f, "μ"),
            Self::Milli => write!(f, "m"),
            Self::Centi => write!(f, "c"),
            Self::Deci => write!(f, "d"),
            Self::Base => write!(f, ""),
            Self::Deca => write!(f, "da"),
            Self::Hecto => write!(f, "h"),
            Self::Kilo => write!(f, "k"),
            Self::Mega => write!(f, "M"),
            Self::Giga => write!(f, "G"),
            Self::Tera => write!(f, "T"),
            Self::Peta => write!(f, "P"),
            Self::Exa => write!(f, "E"),
            Self::Zetta => write!(f, "Z"),
            Self::Yotta => write!(f, "Y"),
            Self::Undefined => write!(f, ""),
        }
    }
}

/// Diffrent units of flow the device can be calibrated to
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Units {
    NormLiter,
    StandardLiter,
    LiterLiquid,
    Gram,
    Pascal,
    Bar,
    MeterH20,
    InchH20,
    Undefined,
}

impl From<u8> for Units {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::NormLiter,
            1 => Self::StandardLiter,
            8 => Self::LiterLiquid,
            9 => Self::Gram,
            16 => Self::Pascal,
            17 => Self::Bar,
            18 => Self::MeterH20,
            19 => Self::InchH20,
            _ => Self::Undefined,
        }
    }
}

impl From<Units> for u8 {
    fn from(value: Units) -> Self {
        match value {
            Units::NormLiter => 0,
            Units::StandardLiter => 1,
            Units::LiterLiquid => 8,
            Units::Gram => 9,
            Units::Pascal => 16,
            Units::Bar => 17,
            Units::MeterH20 => 18,
            Units::InchH20 => 19,
            Units::Undefined => 0xFF,
        }
    }
}

impl Display for Units {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NormLiter | Self::StandardLiter | Self::LiterLiquid => write!(f, "l"),
            Self::Gram => write!(f, "g"),
            Self::Pascal => write!(f, "Pa"),
            Self::Bar => write!(f, "bar"),
            Self::MeterH20 => write!(f, "mH20"),
            Self::InchH20 => write!(f, "iH20"),
            Self::Undefined => write!(f, ""),
        }
    }
}

/// Timescales for the calibrations
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum TimeBases {
    None,
    Microsecond,
    Milisecond,
    Second,
    Minute,
    Hour,
    Day,
    Undefined,
}

impl From<u8> for TimeBases {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Microsecond,
            2 => Self::Milisecond,
            3 => Self::Second,
            4 => Self::Minute,
            5 => Self::Hour,
            6 => Self::Day,
            _ => Self::Undefined,
        }
    }
}

impl From<TimeBases> for u8 {
    fn from(value: TimeBases) -> Self {
        match value {
            TimeBases::None => 0,
            TimeBases::Microsecond => 1,
            TimeBases::Milisecond => 2,
            TimeBases::Second => 3,
            TimeBases::Minute => 4,
            TimeBases::Hour => 5,
            TimeBases::Day => 6,
            TimeBases::Undefined => 0xFF,
        }
    }
}

impl Display for TimeBases {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, ""),
            Self::Microsecond => write!(f, "/μs"),
            Self::Milisecond => write!(f, "/ms"),
            Self::Second => write!(f, "/s"),
            Self::Minute => write!(f, "/min"),
            Self::Hour => write!(f, "/h"),
            Self::Day => write!(f, "/day"),
            Self::Undefined => write!(f, ""),
        }
    }
}
