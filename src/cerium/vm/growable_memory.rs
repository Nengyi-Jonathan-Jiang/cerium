use super::{CeWord, Pointer};
use crate::cerium::cerium_error::CeriumVMError;
use crate::cerium::memory_buffer::{EndianConversion, MemoryBuffer, MemoryBufferPtr};
use std::mem::size_of;

static mut MAX_SIZE: CeWord = 1 << 12;
fn max_memory() -> CeWord {
    unsafe { MAX_SIZE }
}
#[allow(unused)]
pub unsafe fn config_growable_memory_max_size(size: CeWord) {
    MAX_SIZE = size
}

pub struct GrowableMemoryBlock {
    pub memory: MemoryBuffer,
}

impl GrowableMemoryBlock {
    pub fn capacity(&self) -> CeWord {
        self.memory.size()
    }
}

impl Default for GrowableMemoryBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl GrowableMemoryBlock {
    const INITIAL_MEMORY: CeWord = 1 << 8;

    pub fn new() -> Self {
        let mut memory = MemoryBuffer::new();
        memory.resize(Self::INITIAL_MEMORY as usize);
        GrowableMemoryBlock { memory }
    }

    #[inline(always)]
    pub fn resize_to_fit(&mut self, size: CeWord) {
        if size > max_memory() {
            CeriumVMError::throw_string(
                format!("memory size cannot exceed {} bytes", max_memory()).to_owned(),
            );
        } else {
            if size > self.memory.size() {
                self.memory.resize(usize::next_power_of_two(size as usize));
            }
        }
    }

    #[inline(always)]
    pub fn at<T: EndianConversion>(&mut self, ptr: Pointer) -> MemoryBufferPtr<T> {
        self.resize_to_fit(CeWord::from(ptr) + size_of::<T>() as CeWord);

        self.memory.get(CeWord::from(ptr) as usize)
    }
}
