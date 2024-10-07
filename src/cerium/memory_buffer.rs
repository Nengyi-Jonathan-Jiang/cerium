use crate::cerium::vm::CeWord;

pub struct MemoryBufferPtr<T: Endianness> {
    ptr: *mut T,
}

impl<T: Endianness> MemoryBufferPtr<T> {
    pub unsafe fn new<U>(ptr: *mut U) -> Self {
        MemoryBufferPtr { ptr: ptr.cast() }
    }
    pub fn ptr(&self) -> *mut T {
        self.ptr
    }
    pub unsafe fn write(&mut self, val: T) {
        self.ptr.as_mut().unwrap().clone_from(&val.to_big_endian())
    }
    pub fn get(&mut self) -> T {
        unsafe { T::from_big_endian(self.ptr.cast::<T>().as_mut().unwrap()) }
    }
}

#[derive(Default)]
pub struct MemoryBuffer {
    pub memory: Vec<u8>,
}

impl From<&[u8]> for MemoryBuffer {
    fn from(value: &[u8]) -> Self {
        MemoryBuffer { memory: value.to_vec() }
    }
}

impl MemoryBuffer {
    pub fn new() -> MemoryBuffer { Default::default() }
    pub fn size(&self) -> usize { self.memory.len() }
    
    pub fn resize(&mut self, new_size: usize) {
        self.memory.resize(new_size, 0);
    }
    
    pub fn push(&mut self, byte: u8) {
        self.memory.push(byte);
    }
    
    pub fn extend(&mut self, bytes: &[u8]) {
        self.memory.extend_from_slice(bytes);
    }

    pub fn get<T: Endianness>(&self, ptr: usize) -> MemoryBufferPtr<T> {
        assert!(ptr + size_of::<T>() <= self.memory.len(), "Invalid access of memory buffer");
        unsafe {
            MemoryBufferPtr::new(self.memory.as_ptr().add(ptr) as *mut u8)
        }
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

pub trait Endianness: Sized + Copy {
    fn from_big_endian(value: &Self) -> Self {
        *value
    }

    fn to_big_endian(&self) -> Self {
        *self
    }
}

impl Endianness for u8 {}

impl Endianness for i8 {}

impl Endianness for i16 {
    fn from_big_endian(value: &Self) -> Self {
        Self::from_be(*value)
    }
    fn to_big_endian(&self) -> Self {
        self.to_be()
    }
}

impl Endianness for i32 {
    fn from_big_endian(value: &Self) -> Self {
        Self::from_be(*value)
    }
    fn to_big_endian(&self) -> Self {
        self.to_be()
    }
}

impl Endianness for u32 {
    fn from_big_endian(value: &Self) -> Self {
        Self::from_be(*value)
    }
    fn to_big_endian(&self) -> Self {
        self.to_be()
    }
}

impl Endianness for f32 {}