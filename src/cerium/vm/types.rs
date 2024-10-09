use std::fmt::{Debug, Formatter};
use std::ops::{Add, Sub};

pub type CeWord = u32;
pub type CeInt32 = i32;
pub type CeInt16 = i16;
pub type CeInt8 = i8;
pub type CeFloat = f32;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
#[repr(transparent)]
pub struct Size(CeWord);

impl Add for Size {
    type Output = Size;

    fn add(self, rhs: Self) -> Self::Output {
        Size(self.0 + rhs.0)
    }
}

impl From<CeWord> for Size {
    fn from(size: CeWord) -> Size {
        Size(size)
    }
}

impl From<Size> for CeWord {
    fn from(value: Size) -> Self {
        value.0
    }
}

impl Sub for Size {
    type Output = Size;

    fn sub(self, rhs: Self) -> Self::Output {
        Size(self.0 - rhs.0)
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
#[repr(transparent)]
pub struct Pointer(CeWord);

impl Pointer {
    pub fn new(value: CeWord) -> Self { Pointer(value) }
}

impl Add<Size> for Pointer {
    type Output = Pointer;

    fn add(self, rhs: Size) -> Self::Output {
        Pointer(self.0 + rhs.0)
    }
}

impl Sub<Pointer> for Pointer {
    type Output = Size;

    fn sub(self, rhs: Pointer) -> Self::Output {
        (self.0 - rhs.0).into()
    }
}

impl From<CeWord> for Pointer {
    fn from(size: CeWord) -> Pointer {
        Pointer(size)
    }
}

impl From<Pointer> for CeWord {
    fn from(value: Pointer) -> Self {
        value.0
    }
}

impl Debug for Pointer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:08x}", self.0)
    }
}