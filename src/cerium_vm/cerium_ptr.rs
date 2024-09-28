use std::ops::{Add, Sub};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct CeriumSize(usize);

impl Add for CeriumSize {
    type Output = CeriumSize;

    fn add(self, rhs: Self) -> Self::Output {
        CeriumSize(self.0 + rhs.0)
    }
}

impl From<usize> for CeriumSize {
    fn from(size: usize) -> CeriumSize {
        CeriumSize(size)
    }
}

impl From<CeriumSize> for usize {
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

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub struct CeriumPtr(usize);

impl Add<CeriumSize> for CeriumPtr {
    type Output = CeriumPtr;

    fn add(self, rhs: CeriumSize) -> Self::Output {
        CeriumPtr(self.0 + rhs.0)
    }
}

impl From<usize> for CeriumPtr {
    fn from(size: usize) -> CeriumPtr {
        CeriumPtr(size)
    }
}

impl From<CeriumPtr> for usize {
    fn from(value: CeriumPtr) -> Self {
        value.0
    }
}