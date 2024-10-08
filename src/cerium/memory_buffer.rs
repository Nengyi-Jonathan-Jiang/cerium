use crate::cerium::vm::CeWord;

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
    pub fn new() -> MemoryBuffer { Default::default() }
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
        debug_assert!(ptr + size_of::<T>() <= self.memory.len(), "Invalid access of memory buffer");
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

pub trait EndianConversion: Sized + Copy {
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