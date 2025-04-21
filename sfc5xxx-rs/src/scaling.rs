#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scale {
    Normilized,
    PhysicalValue,
    UserDefined
}
