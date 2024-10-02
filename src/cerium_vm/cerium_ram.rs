use super::growable_memory::GrowableMemoryBlock;
use crate::cerium_vm::allocator::Allocator;
use crate::cerium_vm::cerium_types::{CeriumPtr, CeriumSize};
use crate::cerium_vm::{CeWord, CeriumType};
use crate::cerium_vm::memory_buffer::MemoryBufferPtr;

#[derive(Default)]
pub struct CeriumRAM {
    stack_memory: GrowableMemoryBlock,
    heap_memory: GrowableMemoryBlock,
    allocator: Allocator,
}

impl CeriumRAM {
    const HEAP_PTR_BIT: CeWord = (1 << (size_of::<CeWord>() * 8 - 1)) as CeWord;

    fn is_heap_ptr(ptr: CeriumPtr) -> bool {
        (CeWord::from(ptr) & Self::HEAP_PTR_BIT) != 0
    }

    fn ptr_to_mem_ptr(ptr: CeriumPtr) -> CeriumPtr {
        (CeWord::from(ptr) & !Self::HEAP_PTR_BIT).into()
    }

    fn mem_ptr_to_ptr(ptr: CeriumPtr, is_heap: bool) -> CeriumPtr {
        if is_heap {
            (CeWord::from(ptr) | Self::HEAP_PTR_BIT).into()
        } else {
            ptr
        }
    }

    fn resize_mem_to_fit(&mut self, ptr: CeriumPtr) -> Result<(), String> {
        let mem_ptr = Self::ptr_to_mem_ptr(ptr);
        if Self::is_heap_ptr(mem_ptr) {
            self.heap_memory.resize_to_fit(mem_ptr.into())
        } else {
            self.stack_memory.resize_to_fit(mem_ptr.into())
        }
    }

    pub fn at<T: CeriumType>(&mut self, ptr: CeriumPtr) -> Result<MemoryBufferPtr<T>, String> {
        let mem_ptr = Self::ptr_to_mem_ptr(ptr);
        if Self::is_heap_ptr(ptr) {
            self.heap_memory.at(mem_ptr.into())
        } else {
            self.stack_memory.at(mem_ptr.into())
        }
    }

    pub fn allocate(&mut self, size: CeWord) -> Result<CeriumPtr, String> {
        let size: CeriumSize = size.into();

        if CeWord::from(size) == 0 {
            return Err("CeriumVM error: allocation must not be empty".to_owned());
        }

        let heap_ptr = self.allocator.allocate(size.into());
        if let Err(err) = self.resize_mem_to_fit(heap_ptr + size) {
            return Err(err);
        }

        Ok(Self::mem_ptr_to_ptr(heap_ptr, true))
    }

    pub fn deallocate(&mut self, ptr: CeriumPtr) -> Result<(), String> {
        if !Self::is_heap_ptr(ptr) {
            return Err("CeriumVM Error: Attempting to deallocate non-heap pointer".to_owned());
        }
        let heap_ptr = Self::ptr_to_mem_ptr(ptr);
        self.allocator.deallocate(heap_ptr)
    }
    
    pub fn memcpy(&mut self, src: CeriumPtr, dst: CeriumPtr, length: CeriumSize) -> Result<(), String> {
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