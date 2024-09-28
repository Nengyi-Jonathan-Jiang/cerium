use crate::cerium_vm::allocator::Allocator;
use crate::cerium_vm::cerium_ptr::{CeriumPtr, CeriumSize};
use super::growable_memory::GrowableMemoryBlock;

pub struct CeriumMemory {
    stack_memory: GrowableMemoryBlock,
    heap_memory: GrowableMemoryBlock,
    allocator: Allocator,
}

impl CeriumMemory {
    const HEAP_PTR_BIT: usize = 1 << (size_of::<usize>() * 2 - 1);

    pub fn new() -> Self {
        CeriumMemory {
            stack_memory: GrowableMemoryBlock::new(),
            heap_memory: GrowableMemoryBlock::new(),
            allocator: Allocator::new(),
        }
    }

    fn is_heap_ptr(ptr: CeriumPtr) -> bool { usize::from(ptr) & Self::HEAP_PTR_BIT == 0 }

    fn ptr_to_mem_ptr(ptr: CeriumPtr) -> CeriumPtr {
        (usize::from(ptr) & !Self::HEAP_PTR_BIT).into()
    }

    fn mem_ptr_to_ptr(ptr: CeriumPtr, is_heap: bool) -> CeriumPtr {
        if is_heap {
            (usize::from(ptr) | Self::HEAP_PTR_BIT).into()
        } else { 
            ptr 
        }
    }

    pub fn at<T>(&mut self, ptr: CeriumPtr) -> Result<&mut T, String> {
        let mem_ptr = Self::ptr_to_mem_ptr(ptr);
        if Self::is_heap_ptr(ptr) {
            self.heap_memory.at(mem_ptr.into())
        } else {
            self.stack_memory.at(mem_ptr.into())
        }
    }

    pub fn allocate(&mut self, size: usize) -> Result<CeriumPtr, String> {
        let size: CeriumSize = size.into();
        let heap_ptr = self.allocator.allocate(size.into());
        if let Err(err) = self.heap_memory.resize_to_fit((heap_ptr + size).into()) {
            return Err(err);
        }

        Ok((usize::from(heap_ptr) | Self::HEAP_PTR_BIT).into())
    }

    pub fn deallocate(&mut self, ptr: CeriumPtr) {
        
    }
}