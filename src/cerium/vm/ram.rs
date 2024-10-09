use super::allocator::Allocator;
use super::growable_memory::GrowableMemoryBlock;
use super::types::{Pointer, Size};
use super::CeWord;
use crate::cerium::memory_buffer::{EndianConversion, MemoryBufferPtr};

#[derive(Default)]
pub struct RAM {
    stack_memory: GrowableMemoryBlock,
    heap_memory: GrowableMemoryBlock,
    allocator: Allocator,
}

impl RAM {
    const HEAP_PTR_BIT: CeWord = (1 << (size_of::<CeWord>() * 8 - 1)) as CeWord;

    fn is_heap_ptr(ptr: Pointer) -> bool {
        (CeWord::from(ptr) & Self::HEAP_PTR_BIT) != 0
    }

    fn ptr_to_mem_ptr(ptr: Pointer) -> Pointer {
        (CeWord::from(ptr) & !Self::HEAP_PTR_BIT).into()
    }

    fn mem_ptr_to_ptr(ptr: Pointer, is_heap: bool) -> Pointer {
        if is_heap {
            (CeWord::from(ptr) | Self::HEAP_PTR_BIT).into()
        } else {
            ptr
        }
    }

    fn resize_mem_to_fit(&mut self, ptr: Pointer) -> Result<(), String> {
        let mem_ptr = Self::ptr_to_mem_ptr(ptr);
        if Self::is_heap_ptr(mem_ptr) {
            self.heap_memory.resize_to_fit(mem_ptr.into())
        } else {
            self.stack_memory.resize_to_fit(mem_ptr.into())
        }
    }

    pub fn at<T: EndianConversion>(&mut self, ptr: Pointer) -> Result<MemoryBufferPtr<T>, String> {
        let mem_ptr = Self::ptr_to_mem_ptr(ptr);
        if Self::is_heap_ptr(ptr) {
            self.heap_memory.at(mem_ptr.into())
        } else {
            self.stack_memory.at(mem_ptr.into())
        }
    }

    pub fn allocate(&mut self, size: CeWord) -> Result<Pointer, String> {
        let size: Size = size.into();

        if CeWord::from(size) == 0 {
            return Err("CeriumVM error: allocation must not be empty".to_owned());
        }

        let heap_ptr = self.allocator.allocate(size.into());
        if let Err(err) = self.resize_mem_to_fit(heap_ptr + size) {
            return Err(err);
        }

        Ok(Self::mem_ptr_to_ptr(heap_ptr, true))
    }

    pub fn deallocate(&mut self, ptr: Pointer) -> Result<(), String> {
        if !Self::is_heap_ptr(ptr) {
            return Err("CeriumVM Error: Attempting to deallocate non-heap pointer".to_owned());
        }
        let heap_ptr = Self::ptr_to_mem_ptr(ptr);
        self.allocator.deallocate(heap_ptr)
    }
    
    pub fn memcpy(&mut self, src: Pointer, dst: Pointer, length: Size) -> Result<(), String> {
        if let Err(err) = self.resize_mem_to_fit(src + length) {
            return Err(err);
        }
        if let Err(err) = self.resize_mem_to_fit(dst + length) {
            return Err(err);
        }

        let dst_ptr = self.at::<i8>(dst)?.ptr() as *mut u8;
        let src_ptr = self.at::<i8>(src)?.ptr() as *const u8;
        
        unsafe {
            std::ptr::copy(src_ptr, dst_ptr, CeWord::from(length) as usize);
        }
        
        Ok(())
    }
}