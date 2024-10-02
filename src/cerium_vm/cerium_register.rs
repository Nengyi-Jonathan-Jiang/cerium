use crate::cerium_vm::{CeWord, CeriumType};
use crate::cerium_vm::memory_buffer::{MemoryBufferPtr};

#[derive(Default)]
pub struct CeriumRegister {
    value: CeWord,
}

impl CeriumRegister {
    pub fn get<T: CeriumType>(&mut self) -> MemoryBufferPtr<T> {
        unsafe {
            MemoryBufferPtr::new((&mut self.value) as *mut CeWord)
        }
    }
}