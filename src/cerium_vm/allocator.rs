use std::collections::{BTreeMap, BTreeSet};
use crate::cerium_vm::cerium_ptr::{CeriumPtr, CeriumSize};
use crate::util::multimap::OrderedSetMultiMap;

#[derive(Copy, Clone)]
struct MemoryBlock {
    start_ptr: CeriumPtr,
    size: CeriumSize,
}

impl MemoryBlock {
    fn end_ptr(&self) -> CeriumPtr {
        self.start_ptr + self.size
    }
}

pub struct Allocator {
    used_blocks: BTreeMap<CeriumPtr, CeriumSize>,
    available_blocks: OrderedSetMultiMap<CeriumSize, CeriumPtr>,

    available_block_before: BTreeMap<CeriumPtr, CeriumPtr>,
    available_block_size: BTreeMap<CeriumPtr, CeriumSize>,
    last_heap_ptr: CeriumPtr,
}

impl Allocator {
    pub fn new() -> Allocator {
        Allocator {
            used_blocks: Default::default(),
            available_blocks: Default::default(),
            available_block_before: Default::default(),
            available_block_size: Default::default(),
            last_heap_ptr: 0.into(),
        }
    }

    /// Retrieves 
    fn get_used_block_for(&self, start_ptr: CeriumPtr) -> Option<MemoryBlock> {
        if !self.is_valid_pointer(start_ptr) {
            return None;
        }

        let size = *self.used_blocks.get(&start_ptr).unwrap();

        Some(MemoryBlock { start_ptr, size })
    }

    /// Marks a memory block as available and merges it with adjacent available blocks
    fn mark_block_available(&mut self, mut block: MemoryBlock) {
        self.used_blocks.remove(&block.start_ptr);

        self.available_blocks.insert(block.size, block.start_ptr);

        self.available_block_before.insert(block.end_ptr(), block.start_ptr);
        self.available_block_size.insert(block.start_ptr, block.size);

        // Merge with block before
        if let Some(start_ptr) = self.available_block_before.get(&block.start_ptr).cloned() {
            let size = self.available_block_size.get(&start_ptr).unwrap().clone();

            block = self.merge_free_blocks(
                MemoryBlock { start_ptr, size },
                block,
            );
        }

        // Merge with block after
        if let Some(size) = self.available_block_size.get(&block.end_ptr()).cloned() {
            let start_ptr = block.end_ptr();
            self.merge_free_blocks(
                block,
                MemoryBlock { start_ptr, size },
            );
        }
    }

    fn merge_free_blocks(
        &mut self,
        block1: MemoryBlock,
        block2: MemoryBlock,
    ) -> MemoryBlock {
        let start_ptr = block1.start_ptr;
        let end_ptr = block2.end_ptr();
        let middle_ptr = block1.end_ptr();
        let size = block1.size + block2.size;
        if middle_ptr != block2.start_ptr {
            panic!("JeVM Internal Error: can only merged consecutive available blocks");
        }

        self.available_block_size.insert(start_ptr, size);
        self.available_block_size.remove(&middle_ptr);
        self.available_block_before.remove(&middle_ptr);
        self.available_block_before.insert(end_ptr, start_ptr);

        self.remove_block(&block1);
        self.remove_block(&block2);

        self.available_blocks.insert(size, start_ptr);

        MemoryBlock { start_ptr, size }
    }

    fn remove_block(&mut self, block: &MemoryBlock) {
        self.available_blocks.remove(block.size, block.start_ptr);
    }

    fn retrieve_available_block_with_minimum_size(&mut self, min_size: CeriumSize) -> Option<MemoryBlock> {
        if let Some(size) = self.available_blocks.next_higher_key(min_size).copied() {
            if let Some(start_ptr) = self.available_blocks.remove_first_value_for(size) {
                return Some(MemoryBlock { start_ptr, size });
            }
        }

        None
    }

    pub fn allocate(&mut self, alloc_size: CeriumSize) -> CeriumPtr {
        let start_ptr: CeriumPtr;

        // Try to find a free block of the right size
        if let Some(block) = self.retrieve_available_block_with_minimum_size(alloc_size) {
            self.mark_block_available(block);
            start_ptr = block.start_ptr;

            // Split the block
            if block.size > alloc_size {
                self.mark_block_available(
                    MemoryBlock {
                        start_ptr: block.start_ptr + alloc_size,
                        size: block.size - alloc_size,
                    }
                );
            }
        } else {
            // Allocate a new space at the end of the heap
            start_ptr = self.last_heap_ptr;
            self.last_heap_ptr = self.last_heap_ptr + alloc_size;
        }

        // Update sizes map
        self.used_blocks.insert(start_ptr, alloc_size);

        start_ptr
    }

    pub fn deallocate(&mut self, ptr: CeriumPtr) {
        if let Some(used_block) = self.get_used_block_for(ptr) {
            self.mark_block_available(used_block);
        } else {
            panic!("JeVM Error: invalid pointer to deallocate");
        }
    }

    pub fn is_valid_pointer(&self, ptr: CeriumPtr) -> bool {
        self.used_blocks.contains_key(&ptr)
    }
}