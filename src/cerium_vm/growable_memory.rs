use crate::cerium_vm::{CeWord, CeriumPtr, CeriumType};
use crate::cerium_vm::memory_buffer::{MemoryBuffer, MemoryBufferPtr};

pub struct GrowableMemoryBlock {
    pub memory: MemoryBuffer,
}

impl Default for GrowableMemoryBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl GrowableMemoryBlock {
    const INITIAL_MEMORY: CeWord = 1 << 8;
    const MAX_MEMORY: CeWord = 1 << 16;

    pub fn new() -> Self {
        let mut memory = MemoryBuffer::new();
        memory.resize(Self::INITIAL_MEMORY as usize);
        GrowableMemoryBlock { memory }
    }

    pub fn resize_to_fit(&mut self, size: CeWord) -> Result<(), String> {
        if size > Self::MAX_MEMORY {
            Err(format!(
                "CeriumVM error: memory size cannot exceed {} bytes",
                Self::MAX_MEMORY
            ).to_owned())
        } else {
            if size > self.memory.size() as CeWord {
                self.memory.resize(usize::next_power_of_two(size as usize));
            }

            Ok(())
        }
    }

    pub fn at<T: CeriumType>(&mut self, ptr: CeriumPtr) -> Result<MemoryBufferPtr<T>, String> {
        match self.resize_to_fit(CeWord::from(ptr) + size_of::<T>() as CeWord) {
            Ok(_) => unsafe {
                Ok(self.memory.get(CeWord::from(ptr) as usize))
            }
            Err(err) => {
                Err(err)
            }
        }
    }
}