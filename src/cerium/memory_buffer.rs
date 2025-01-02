use crate::cerium::cerium_error::CeriumVMError;
use crate::cerium::vm::{CeFloat, CeInt16, CeInt32, CeInt8, CeWord};
use std::any::TypeId;
use std::fmt::{Debug, Display};
use std::mem::size_of;
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::ptr::from_ref;

#[repr(transparent)]
pub struct MemoryBufferPtr<T: EndianConversion> {
    ptr: *mut T,
}

impl<T: EndianConversion> MemoryBufferPtr<T> {
    pub unsafe fn new<U>(ptr: *mut U) -> Self {
        MemoryBufferPtr { ptr: ptr.cast() }
    }
    #[inline(always)]
    pub fn ptr(&self) -> *mut T {
        self.ptr
    }
    #[inline(always)]
    pub unsafe fn write(&mut self, val: T) {
        self.ptr.write(val.to_big_endian())
    }
    #[inline(always)]
    pub fn get(&mut self) -> T {
        unsafe { T::from_big_endian(&self.ptr.cast::<T>().read()) }
    }
}

pub struct MemoryBuffer {
    memory: Vec<u8>,
    size: CeWord,
    ptr: *mut u8,
}

impl Default for MemoryBuffer {
    fn default() -> Self {
        MemoryBuffer::from(vec![])
    }
}

impl<T: Into<Vec<u8>>> From<T> for MemoryBuffer {
    fn from(value: T) -> Self {
        let memory: Vec<u8> = value.into();
        MemoryBuffer {
            ptr: memory.as_ptr() as *mut u8,
            size: memory.len() as CeWord,
            memory,
        }
    }
}

impl MemoryBuffer {
    pub fn new() -> MemoryBuffer {
        Default::default()
    }
    pub fn size(&self) -> CeWord {
        self.size
    }

    #[inline(always)]
    fn update(&mut self) {
        self.size = self.memory.len() as CeWord;
        self.ptr = self.memory.as_ptr() as *mut u8;
    }

    pub fn resize(&mut self, new_size: usize) {
        self.memory.resize(new_size, 0);
        self.update();
    }

    pub fn push(&mut self, byte: u8) {
        self.memory.push(byte);
        self.update();
    }

    pub fn extend(&mut self, bytes: &[u8]) {
        self.memory.extend_from_slice(bytes);
        self.update();
    }

    #[inline(always)]
    pub fn get<T: EndianConversion>(&self, ptr: usize) -> MemoryBufferPtr<T> {
        debug_assert!(
            ptr + size_of::<T>() <= self.memory.len(),
            "Invalid access of memory buffer"
        );
        unsafe { MemoryBufferPtr::new(self.ptr.add(ptr)) }
    }
}

impl Into<Box<[u8]>> for MemoryBuffer {
    fn into(self) -> Box<[u8]> {
        self.memory.into_boxed_slice()
    }
}

impl<'a> Into<&'a [u8]> for &'a MemoryBuffer {
    fn into(self) -> &'a [u8] {
        self.memory.as_slice()
    }
}

pub trait EndianConversion: Sized + Copy + 'static {
    fn from_big_endian(value: &Self) -> Self {
        *value
    }

    fn to_big_endian(&self) -> Self {
        *self
    }
}

impl EndianConversion for u8 {}

impl EndianConversion for i8 {}

impl EndianConversion for i16 {
    fn from_big_endian(value: &Self) -> Self {
        Self::from_be(*value)
    }
    fn to_big_endian(&self) -> Self {
        self.to_be()
    }
}

impl EndianConversion for i32 {
    fn from_big_endian(value: &Self) -> Self {
        Self::from_be(*value)
    }
    fn to_big_endian(&self) -> Self {
        self.to_be()
    }
}

impl EndianConversion for u32 {
    fn from_big_endian(value: &Self) -> Self {
        Self::from_be(*value)
    }
    fn to_big_endian(&self) -> Self {
        self.to_be()
    }
}

impl EndianConversion for f32 {}

pub trait CeriumPrimitiveType:
    EndianConversion
    + Debug
    + Display
    + From<CeInt8>
    + PartialOrd
    + Mul<Output = Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
{
    fn cast_to_primitive<T: CeriumPrimitiveType>(self) -> T;

    fn xor(a: Self, b: Self) -> Self;
    fn and(a: Self, b: Self) -> Self;
    fn or(a: Self, b: Self) -> Self;
    fn shl(a: Self, b: Self) -> Self;
    fn shr(a: Self, b: Self) -> Self;
    fn add(a: Self, b: Self) -> Self;
    fn sub(a: Self, b: Self) -> Self;
    fn mul(a: Self, b: Self) -> Self;
    fn div(a: Self, b: Self) -> Self;
    fn rem(a: Self, b: Self) -> Self;
    fn neg(a: Self) -> Self;
    fn not(a: Self) -> Self;
}

macro_rules! do_conversion_for {
    ($value: ident : $type_id: ident as $ty: ty) => {
        if $type_id == TypeId::of::<$ty>() {
            let res = $value as $ty;
            unsafe {
                let res_ptr: *const T = std::mem::transmute(from_ref(&res));
                return *res_ptr;
            }
        }
    };
}

macro_rules! impl_cerium_primitive_type_for {
    ($ty: ty) => {
        impl CeriumPrimitiveType for $ty {
            fn cast_to_primitive<T: CeriumPrimitiveType>(self) -> T {
                let ty = TypeId::of::<T>();

                do_conversion_for!(self: ty as CeInt8);
                do_conversion_for!(self: ty as CeInt16);
                do_conversion_for!(self: ty as CeInt32);
                do_conversion_for!(self: ty as CeFloat);

                CeriumVMError::throw_str("Invalid cast between CeriumPrimitiveTypes");
            }

            fn xor(a: Self, b: Self) -> Self { a ^ b }
            fn and(a: Self, b: Self) -> Self { a & b }
            fn or(a: Self, b: Self) -> Self { a | b }
            fn shl(a: Self, b: Self) -> Self { a << b }
            fn shr(a: Self, b: Self) -> Self { a >> b }
            fn add(a: Self, b: Self) -> Self { a + b }
            fn sub(a: Self, b: Self) -> Self { a - b }
            fn mul(a: Self, b: Self) -> Self { a * b }
            fn div(a: Self, b: Self) -> Self { a / b }
            fn rem(a: Self, b: Self) -> Self { (a % b + b) % b }
            fn neg(a: Self) -> Self { -a }
            fn not(a: Self) -> Self { !a }
        }
    }
}

impl_cerium_primitive_type_for!(CeInt8);
impl_cerium_primitive_type_for!(CeInt16);
impl_cerium_primitive_type_for!(CeInt32);

impl CeriumPrimitiveType for CeFloat {
    fn cast_to_primitive<T: CeriumPrimitiveType>(self) -> T {
        let ty = TypeId::of::<T>();

        do_conversion_for!(self: ty as CeInt8);
        do_conversion_for!(self: ty as CeInt16);
        do_conversion_for!(self: ty as CeInt32);
        do_conversion_for!(self: ty as CeFloat);

        CeriumVMError::throw_str("Invalid cast between CeriumPrimitiveTypes");
    }

    fn xor(_: Self, _: Self) -> Self {
        CeriumVMError::throw_str("Bitwise xor cannot be applied to float")
    }
    fn and(_: Self, _: Self) -> Self {
        CeriumVMError::throw_str("Bitwise and cannot be applied to float")
    }
    fn or(_: Self, _: Self) -> Self {
        CeriumVMError::throw_str("Bitwise or cannot be applied to float")
    }
    fn shl(_: Self, _: Self) -> Self {
        CeriumVMError::throw_str("Bitwise left shift cannot be applied to float")
    }
    fn shr(_: Self, _: Self) -> Self {
        CeriumVMError::throw_str("Bitwise right shift cannot be applied to float")
    }
    fn add(a: Self, b: Self) -> Self {
        a + b
    }
    fn sub(a: Self, b: Self) -> Self {
        a - b
    }
    fn mul(a: Self, b: Self) -> Self {
        a * b
    }
    fn div(a: Self, b: Self) -> Self {
        a / b
    }
    fn rem(a: Self, b: Self) -> Self {
        (a % b + b) % b
    }
    fn neg(a: Self) -> Self {
        -a
    }
    fn not(_: Self) -> Self {
        CeriumVMError::throw_str("Bitwise not cannot be applied to float")
    }
}
