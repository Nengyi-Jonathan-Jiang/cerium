use super::{CeWord};
use crate::cerium::memory_buffer::{Endianness, MemoryBufferPtr};

#[derive(Default)]
pub struct CeriumRegister {
    value: CeWord,
}

impl CeriumRegister {
    pub fn get<T: Endianness>(&mut self) -> MemoryBufferPtr<T> {
        unsafe {
            MemoryBufferPtr::new((&mut self.value) as *mut CeWord)
        }
    }
}