use super::allocator::Allocator;
use super::growable_memory::GrowableMemoryBlock;
use super::types::{Pointer, Size};
use super::CeWord;
use crate::cerium::cerium_error::CeriumVMError;
use crate::cerium::memory_buffer::{EndianConversion, MemoryBufferPtr};
use std::mem::size_of;

#[derive(Default)]
pub struct RAM {
    stack_memory: GrowableMemoryBlock,
    heap_memory: GrowableMemoryBlock,
    allocator: Allocator,
}

impl RAM {
    const HEAP_PTR_BIT: CeWord = (1 << (size_of::<CeWord>() * 8 - 1)) as CeWord;

    pub fn stack_capacity(&self) -> CeWord {
        self.stack_memory.capacity()
    }
    pub fn heap_capacity(&self) -> CeWord {
        self.heap_memory.capacity()
    }
    
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

    fn resize_mem_to_fit(&mut self, ptr: Pointer) {
        let mem_ptr = Self::ptr_to_mem_ptr(ptr);
        if Self::is_heap_ptr(mem_ptr) {
            self.heap_memory.resize_to_fit(mem_ptr.into())
        } else {
            self.stack_memory.resize_to_fit(mem_ptr.into())
        }
    }

    pub fn at<T: EndianConversion>(&mut self, ptr: Pointer) -> MemoryBufferPtr<T> {
        let mem_ptr = Self::ptr_to_mem_ptr(ptr);
        if Self::is_heap_ptr(ptr) {
            self.heap_memory.at(mem_ptr.into())
        } else {
            self.stack_memory.at(mem_ptr.into())
        }
    }

    pub fn allocate(&mut self, size: CeWord) -> Pointer {
        let size: Size = size.into();

        if CeWord::from(size) == 0 {
            CeriumVMError::throw_str("allocation must not be empty");
        }

        let heap_ptr = self.allocator.alloc(size.into());
        self.resize_mem_to_fit(heap_ptr + size);

        Self::mem_ptr_to_ptr(heap_ptr, true)
    }

    pub fn deallocate(&mut self, ptr: Pointer) {
        if !Self::is_heap_ptr(ptr) {
            CeriumVMError::throw_str("Attempting to deallocate non-heap pointer");
        }
        let heap_ptr = Self::ptr_to_mem_ptr(ptr);
        self.allocator.free(heap_ptr)
    }
    
    pub fn get_allocation_size(&self, ptr: Pointer) -> Size {
        if !Self::is_heap_ptr(ptr) {
            CeriumVMError::throw_str("Querying size of non-heap pointer");
        }
        let heap_ptr = Self::ptr_to_mem_ptr(ptr);
        
        self.allocator.get_allocation_size(heap_ptr)
    }

    pub fn memcpy(&mut self, src: Pointer, dst: Pointer, length: Size) {
        self.resize_mem_to_fit(src + length);
        self.resize_mem_to_fit(dst + length);
        
        let dst_ptr = self.at::<i8>(dst).ptr() as *mut u8;
        let src_ptr = self.at::<i8>(src).ptr() as *const u8;

        unsafe {
            std::ptr::copy(src_ptr, dst_ptr, CeWord::from(length) as usize);
        }
    }

    pub unsafe fn write(
        &mut self,
        dst: Pointer,
        data: impl IntoIterator<Item = u8>,
    ) {
        let data = data.into_iter().collect::<Box<[_]>>();
        let length = Size::from(data.len() as CeWord);

        self.resize_mem_to_fit(dst + length);

        let dst_ptr = self.at::<i8>(dst).ptr() as *mut u8;
        let src_ptr = data.as_ptr();

        unsafe {
            std::ptr::copy(src_ptr, dst_ptr, CeWord::from(length) as usize);
        }
    }
}
