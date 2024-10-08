use super::CeWord;
use crate::cerium::memory_buffer::{EndianConversion, MemoryBufferPtr};

#[derive(Default)]
pub struct CeriumRegister {
    value: CeWord,
}

impl CeriumRegister {
    #[inline(always)]
    pub fn get<T: EndianConversion>(&mut self) -> MemoryBufferPtr<T> {
        unsafe {
            MemoryBufferPtr::new((&mut self.value) as *mut CeWord)
        }
    }
}