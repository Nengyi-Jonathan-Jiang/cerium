use crate::cerium_vm::cerium_types::{CeriumPtr, CeriumSize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Debug, Formatter};
use crate::cerium_vm::CeWord;

#[derive(Copy, Clone, Debug)]
struct MemorySpan {
    start: CeriumPtr,
    end: CeriumPtr,
}

impl MemorySpan {
    fn size(&self) -> CeriumSize {
        self.end - self.start
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum MemoryBlockStatus {
    USED,
    FREE,
}

#[derive(Copy, Clone, Debug)]
struct MemoryBlockInfo {
    span: MemorySpan,
    status: MemoryBlockStatus,
    prev_block_start_ptr: Option<CeriumPtr>,
}

#[derive(Default)]
pub struct Allocator {
    blocks: BTreeMap<CeriumPtr, MemoryBlockInfo>,

    free_blocks_for_size: FreeBlocksMap,

    last_heap_ptr: CeriumPtr,
}

impl Allocator {
    fn mark_block_free(&mut self, ptr: CeriumPtr) -> MemoryBlockInfo {
        let curr_block = self.blocks.get_mut(&ptr).expect("Internal CeriumVM error: Invalid pointer");
        curr_block.status = MemoryBlockStatus::FREE;

        let curr_block: MemoryBlockInfo = *curr_block;
        self.free_blocks_for_size.insert(curr_block.span.size(), ptr);
        curr_block
    }

    fn mark_block_used(&mut self, ptr: CeriumPtr) -> MemoryBlockInfo {
        let curr_block = self.blocks.get_mut(&ptr).expect("Internal CeriumVM error: Invalid pointer");
        curr_block.status = MemoryBlockStatus::USED;

        let curr_block: MemoryBlockInfo = *curr_block;
        self.free_blocks_for_size.remove(curr_block.span.size(), ptr);
        curr_block
    }

    fn add_block(&mut self, block: MemoryBlockInfo) {
        if block.status == MemoryBlockStatus::FREE {
            self.free_blocks_for_size.insert(block.span.size(), block.span.start);
        }

        self.blocks.insert(block.span.start, block);
    }

    fn remove_block(&mut self, block: MemoryBlockInfo) {
        if block.status == MemoryBlockStatus::FREE {
            self.free_blocks_for_size.remove(block.span.size(), block.span.start);
        }
        self.blocks.remove(&block.span.start);
    }

    /// Marks a memory block as free and merges it with adjacent free blocks
    fn merge_free_block_with_adjacent(&mut self, mut curr_block: MemoryBlockInfo) {
        // If there is a block before this block
        if let Some(prev_block_ptr) = curr_block.prev_block_start_ptr {
            let prev_block: MemoryBlockInfo = self.blocks.get(&prev_block_ptr).cloned().unwrap();
            // We should merge with it if it is free
            if prev_block.status == MemoryBlockStatus::FREE {
                curr_block = self.merge_free_blocks(
                    prev_block,
                    curr_block,
                );
            }
        }

        // If there is a block after this block
        if let Some(next_block) = self.blocks.get(&curr_block.span.end).cloned() {
            // We should merge with it if it is free
            if next_block.status == MemoryBlockStatus::FREE {
                self.merge_free_blocks(
                    curr_block,
                    next_block,
                );
            }
        }
        // Otherwise, we can remove this block  entirely because it is a trailing free block
        else {
            self.remove_block(curr_block);
            return;
        }
    }

    fn merge_free_blocks(
        &mut self,
        block1: MemoryBlockInfo,
        block2: MemoryBlockInfo,
    ) -> MemoryBlockInfo {
        if block1.span.end != block2.span.start
            || block1.status != MemoryBlockStatus::FREE
            || block2.status != MemoryBlockStatus::FREE
        {
            panic!("Internal CeriumVM Error: can only merged consecutive free blocks");
        }

        let start = block1.span.start;
        let end = block2.span.end;

        let merged_block_info = MemoryBlockInfo {
            span: MemorySpan { start, end },
            status: MemoryBlockStatus::FREE,
            prev_block_start_ptr: block1.prev_block_start_ptr,
        };

        // Update the prev pointer of the block after the merged block, if there is one
        if let Some(next_block) = self.blocks.get_mut(&end) {
            next_block.prev_block_start_ptr = Some(start);
        }

        self.remove_block(block1);
        self.remove_block(block2);
        self.add_block(merged_block_info);

        merged_block_info
    }

    fn split_free_block(
        &mut self,
        block: MemoryBlockInfo,
        left_size: CeriumSize,
    ) -> (MemoryBlockInfo, MemoryBlockInfo) {
        if block.status != MemoryBlockStatus::FREE {
            panic!("Internal CeriumVM Error: can only split a free block");
        }

        let start = block.span.start;
        let middle = block.span.start + left_size;
        let end = block.span.end;

        let left_block = MemoryBlockInfo {
            span: MemorySpan { start, end: middle },
            status: MemoryBlockStatus::FREE,
            prev_block_start_ptr: block.prev_block_start_ptr,
        };
        let right_block = MemoryBlockInfo {
            span: MemorySpan { start: middle, end },
            status: MemoryBlockStatus::FREE,
            prev_block_start_ptr: Some(start),
        };

        // Update the prev pointer of the block after the right block, if there is one
        if let Some(next_block) = self.blocks.get_mut(&end) {
            next_block.prev_block_start_ptr = Some(middle);
        }

        self.remove_block(block);
        self.add_block(left_block);
        self.add_block(right_block);

        (left_block, right_block)
    }

    pub fn allocate(&mut self, alloc_size: CeriumSize) -> CeriumPtr {
        // Try to find a free block of the right size
        if let Some(mut block) = self.free_blocks_for_size.get_first_ptr_with_min_size(alloc_size).and_then(|x| self.blocks.get(&x).cloned()) {
            // Split the block
            if block.span.size() > alloc_size {
                block = self.split_free_block(block, alloc_size).0;
            }

            self.mark_block_used(block.span.start);
            return block.span.start;
        }

        // Allocate a new space at the end of the heap
        let start = self.last_heap_ptr;
        let end = start + alloc_size;

        self.add_block(MemoryBlockInfo {
            span: MemorySpan { start, end },
            status: MemoryBlockStatus::USED,
            prev_block_start_ptr: self.blocks.last_key_value().map(|(x, _)| *x),
        });
        self.last_heap_ptr = end;

        start
    }

    pub fn deallocate(&mut self, ptr: CeriumPtr) -> Result<(), String> {
        if let Some(block) = self.blocks.get(&ptr).cloned() {
            if block.status == MemoryBlockStatus::USED {
                let block = self.mark_block_free(ptr);
                self.merge_free_block_with_adjacent(block);
                return Ok(());
            }
        }

        Err("CeriumVM Error: invalid pointer to deallocate".to_owned())
    }
}

impl Debug for Allocator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        macro_rules! write_or_return {
            ($f:expr, $($arg:tt)*) => {
                if let Err(e) = write!($f, $($arg)*) {
                    return Err(e);
                }
            };
        }


        write_or_return!(f, "Memory layout: ");

        for i in 0..self.last_heap_ptr.into() {
            if let Some(block) = self.blocks.get(&i.into()).cloned() {
                let mut size: CeWord = block.span.size().into();
                let status = block.status;

                match status {
                    MemoryBlockStatus::USED => {
                        if size == 1 {
                            write_or_return!(f, "<>");
                            continue;
                        }
                        size -= 2;
                        write_or_return!(f, "<-");
                        for _ in 0..size {
                            write_or_return!(f, "--");
                        }
                        print!("->");
                    }
                    MemoryBlockStatus::FREE => {
                        if size == 1 {
                            write_or_return!(f, "[]");
                            continue;
                        }
                        size -= 2;
                        write_or_return!(f, "[~");
                        for _ in 0..size {
                            write_or_return!(f, "~~");
                        }
                        write_or_return!(f, "~]");
                    }
                }
            }
        }

        writeln!(f)
    }
}

#[derive(Default, Debug)]
pub struct FreeBlocksMap {
    backing_map: BTreeMap<CeriumSize, BTreeSet<CeriumPtr>>,
}

impl FreeBlocksMap
{
    pub fn get_ptrs_with_size(&mut self, size: CeriumSize) -> &mut BTreeSet<CeriumPtr> {
        self.backing_map.entry(size).or_insert_with(BTreeSet::new)
    }

    pub fn insert(&mut self, key: CeriumSize, value: CeriumPtr) {
        self.get_ptrs_with_size(key).insert(value);
    }

    pub fn remove(&mut self, size: CeriumSize, ptr: CeriumPtr) {
        if let Some(set) = self.backing_map.get_mut(&size) {
            set.remove(&ptr);

            if set.is_empty() {
                self.backing_map.remove(&size);
            }
        }
    }

    pub fn get_first_value_for(&mut self, key: CeriumSize) -> Option<CeriumPtr> {
        if let Some(set) = self.backing_map.get_mut(&key) {
            return set.first().cloned();
        }
        None
    }

    pub fn next_higher_key(&self, lower_bound: CeriumSize) -> Option<CeriumSize> {
        let entry = self.backing_map.range(lower_bound..).next();
        if let Some((key, _)) = entry {
            Some(*key)
        } else {
            None
        }
    }

    pub fn get_first_ptr_with_min_size(&self, minimum_size: CeriumSize) -> Option<CeriumPtr> {
        self.backing_map.range(minimum_size..).next()
            .and_then(|(key, _)| self.backing_map.get(key))
            .and_then(|set| set.first().cloned())
    }
}