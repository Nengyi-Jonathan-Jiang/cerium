use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub};
use crate::cerium_vm::memory_buffer::Endianness;

pub type CeWord = u32;
pub type CeInt32 = i32;
pub type CeInt16 = i16;
pub type CeInt8 = i8;
pub type CeFloat = f32;

pub trait CeriumType: Copy + PartialEq + Add + Sub + Mul + Div + Rem + Neg + PartialEq + PartialOrd + From<i8> + Endianness + Display {}

impl CeriumType for CeInt8 {}

impl CeriumType for CeInt16 {}

impl CeriumType for CeInt32 {}

impl CeriumType for CeFloat {}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct CeriumSize(CeWord);

impl Add for CeriumSize {
    type Output = CeriumSize;

    fn add(self, rhs: Self) -> Self::Output {
        CeriumSize(self.0 + rhs.0)
    }
}

impl From<CeWord> for CeriumSize {
    fn from(size: CeWord) -> CeriumSize {
        CeriumSize(size)
    }
}

impl From<CeriumSize> for CeWord {
    fn from(value: CeriumSize) -> Self {
        value.0
    }
}

impl Sub for CeriumSize {
    type Output = CeriumSize;

    fn sub(self, rhs: Self) -> Self::Output {
        CeriumSize(self.0 - rhs.0)
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
pub struct CeriumPtr(CeWord);

impl CeriumPtr {
    pub fn new(value: CeWord) -> Self { CeriumPtr(value) }
}

impl Add<CeriumSize> for CeriumPtr {
    type Output = CeriumPtr;

    fn add(self, rhs: CeriumSize) -> Self::Output {
        CeriumPtr(self.0 + rhs.0)
    }
}

impl Sub<CeriumPtr> for CeriumPtr {
    type Output = CeriumSize;

    fn sub(self, rhs: CeriumPtr) -> Self::Output {
        (self.0 - rhs.0).into()
    }
}

impl From<CeWord> for CeriumPtr {
    fn from(size: CeWord) -> CeriumPtr {
        CeriumPtr(size)
    }
}

impl From<CeriumPtr> for CeWord {
    fn from(value: CeriumPtr) -> Self {
        value.0
    }
}

impl Debug for CeriumPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:08x}", self.0)
    }
}