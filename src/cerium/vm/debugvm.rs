// #![allow(arithmetic_overflow)]

use super::{CeFloat, CeInt16, CeInt32, CeInt8, CeWord, Pointer};
use crate::cerium::memory_buffer::{EndianConversion, MemoryBufferPtr};
use crate::CeriumVM;
use std::any::TypeId;
use std::hint::unreachable_unchecked;
use std::mem::size_of;

#[derive(Default)]
pub struct DebugCeriumVM {
    vm: CeriumVM,
    program_length: usize,
    instruction_ptr: CeWord,
}

impl DebugCeriumVM {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load_program(&mut self, program: impl IntoIterator<Item = u8>) -> Result<(), String> {
        let program = program.into_iter().collect::<Vec<_>>();
        self.program_length = program.len();

        self.vm.load_program(program)
    }

    fn re_sync_ip_with_vm(&mut self) {
        self.instruction_ptr = self.vm.instruction_ptr;
    }

    fn get_next_and_inc_ip<T: EndianConversion>(&mut self) -> T {
        let res: T = self
            .vm
            .memory
            .at(Pointer::from(self.instruction_ptr))
            .unwrap()
            .get();
        // let res = self.program.get::<T>(self.instruction_ptr as usize).get();
        self.instruction_ptr += size_of::<T>() as CeWord;
        res
    }

    #[inline(always)]
    fn get_memory<T: EndianConversion>(&mut self, bits: u8) -> MemoryBufferPtr<T> {
        self.vm.get_memory(bits)
    }

    #[inline(always)]
    fn get_location<T: EndianConversion>(&mut self, bits: u8) -> MemoryBufferPtr<T> {
        self.vm.get_location(bits)
    }

    #[inline(always)]
    fn get_word_for_location(&mut self, bits: u8) -> CeWord {
        self.get_location::<CeInt32>(bits).get() as CeWord
    }

    pub fn execute_next_instruction(&mut self) {
        self.re_sync_ip_with_vm();
        let curr_instruction_byte = self.get_next_and_inc_ip::<u8>();

        if (curr_instruction_byte >> 6) == 3 {
            // Ternary instructions
            let instruction_part = curr_instruction_byte & 0b00001111;
            let type_part = (curr_instruction_byte & 0b00110000) >> 4;

            let _ = self.get_next_and_inc_ip::<u8>();
            let b3 = self.get_next_and_inc_ip::<u8>();

            macro_rules! do_instruction {
                ($type_part: expr, $b2: expr, $b3: expr; $reason: expr) => {
                    match $type_part {
                        0b00 => self.execute_next_instruction_debug::<CeInt8>($b3 >> 4),
                        0b01 => self.execute_next_instruction_debug::<CeInt16>($b3 >> 4),
                        0b10 => self.execute_next_instruction_debug::<CeInt32>($b3 >> 4),
                        0b11 => self.vm.execute_next_instruction(),
                        _ => unsafe { unreachable_unchecked() },
                    }
                };

                ($type_part: expr, $b2: expr, $b3: expr) => {
                    match $type_part {
                        0b00 => self.execute_next_instruction_debug::<CeInt8>($b3 >> 4),
                        0b01 => self.execute_next_instruction_debug::<CeInt16>($b3 >> 4),
                        0b10 => self.execute_next_instruction_debug::<CeInt32>($b3 >> 4),
                        0b11 => self.execute_next_instruction_debug::<CeFloat>($b3 >> 4),
                        _ => unsafe { unreachable_unchecked() },
                    }
                };
                (call $method: ident, $type_part: expr, $b2: expr, $b3: expr) => {
                    match $type_part {
                        0b00 => self.$method::<CeInt8>($b2, $b3),
                        0b01 => self.$method::<CeInt16>($b2, $b3),
                        0b10 => self.$method::<CeInt32>($b2, $b3),
                        0b11 => self.$method::<CeFloat>($b2, $b3),
                        _ => unsafe {
                            unreachable_unchecked();
                        },
                    }
                };
            }

            match instruction_part {
                0b0000 => self.vm.execute_next_instruction(), // NO-OP
                0b0001 => do_instruction!(type_part, b2, b3),
                0b0010 => do_instruction!(type_part, b2, b3),
                0b0011 => do_instruction!(type_part, b2, b3),
                0b0100 | 0b0101 => self.vm.execute_next_instruction(), // NO-OP
                0b0110 => do_instruction!(type_part, b2, b3),
                0b0111 => do_instruction!(type_part, b2, b3),
                0b1000 => self.vm.execute_next_instruction(), // NO-OP
                0b1001 => do_instruction!(type_part, b2, b3),
                0b1010 => do_instruction!(type_part, b2, b3),
                0b1011 => do_instruction!(type_part, b2, b3),
                0b1100 => do_instruction!(type_part, b2, b3),
                0b1101 => do_instruction!(type_part, b2, b3),
                0b1110 => {
                    // CMP. This is hacky because technically CMP has a different layout of bits
                    // than arithmetic instructions, but it's fine because the target location bits
                    // are in the same position
                    do_instruction!(type_part, b2, b3)
                }
                0b1111 => self.execute_next_instruction_and_debug_jmp(),
                _ => unsafe { unreachable_unchecked() },
            }
        } else {
            let instruction_part = curr_instruction_byte >> 4;
            match instruction_part {
                0b0000 => {
                    // MOV
                    let b2 = self.get_next_and_inc_ip::<u8>();
                    let dst_t = curr_instruction_byte & 0b11;

                    unsafe {
                        match dst_t {
                            0b00 => self.execute_next_instruction_debug::<CeInt8>(b2 >> 4),
                            0b01 => self.execute_next_instruction_debug::<CeInt16>(b2 >> 4),
                            0b10 => self.execute_next_instruction_debug::<CeInt32>(b2 >> 4),
                            0b11 => self.execute_next_instruction_debug::<CeFloat>(b2 >> 4),
                            _ => unreachable_unchecked(),
                        }
                    }
                }
                0b0001 => {
                    // LOD8
                    self.get_next_and_inc_ip::<CeInt8>();
                    self.execute_next_instruction_debug::<CeInt8>(curr_instruction_byte);
                }
                0b0010 => {
                    // LOD16
                    self.get_next_and_inc_ip::<CeInt16>();
                    self.execute_next_instruction_debug::<CeInt16>(curr_instruction_byte);
                }
                0b0011 => {
                    // LOD32
                    self.get_next_and_inc_ip::<CeInt32>();
                    self.execute_next_instruction_debug::<CeInt32>(curr_instruction_byte);
                }
                0b0100 => self.vm.execute_next_instruction(), // HALT
                0b0101 => {
                    // MEMCPY
                    let b2 = self.get_next_and_inc_ip::<u8>();
                    let size = self.get_location::<CeInt32>(curr_instruction_byte).get() as CeWord;
                    let src = self.get_location::<CeInt32>(b2 >> 4).get() as CeWord;
                    let dest = self.get_location::<CeInt32>(b2).get() as CeWord;

                    println!(
                        "<Debug> MEMCPY'ed {} bytes from ptr {} to ptr {}",
                        size, src, dest
                    );

                    self.vm.execute_next_instruction();
                }
                0b0110 => {
                    // NEW
                    let b2 = self.get_next_and_inc_ip::<u8>();
                    let size = self.get_location::<CeInt32>(b2 >> 4).get() as CeWord;
                    println!("<Debug> Allocating {} bytes of memory", size);
                    self.execute_next_instruction_debug::<CeInt32>(b2);
                }
                0b0111 => {
                    // DEL
                    let b2 = self.get_next_and_inc_ip::<u8>();
                    let src = self.get_location::<CeInt32>(b2 >> 4).get() as CeWord;

                    // Debug for deallocate ptr src
                    match self.vm.memory.get_allocation_size(Pointer::from(src)) {
                        Ok(size) => {
                            println!(
                                "<Debug> Deallocated ptr {} (size {})",
                                src,
                                Into::<CeWord>::into(size)
                            );
                        }
                        Err(..) => {
                            println!(
                                "<Debug> Trying to deallocate ptr {} will probably result in error",
                                src
                            );
                        }
                    }
                }
                0b1000 => {
                    // NEG
                    let type_part = (curr_instruction_byte >> 2) & 0b11;
                    let b2 = self.get_next_and_inc_ip::<u8>();

                    match type_part {
                        0b00 => self.execute_next_instruction_debug::<CeInt8>(b2),
                        0b01 => self.execute_next_instruction_debug::<CeInt16>(b2),
                        0b10 => self.execute_next_instruction_debug::<CeInt32>(b2),
                        0b11 => self.execute_next_instruction_debug::<CeFloat>(b2),
                        _ => unsafe { unreachable_unchecked() },
                    }
                }
                0b1001 => {
                    // Bitwise negation
                    let type_part = (curr_instruction_byte >> 2) & 0b11;
                    let b2 = self.get_next_and_inc_ip::<u8>();

                    match type_part {
                        0b00 => self.execute_next_instruction_debug::<CeInt8>(b2),
                        0b01 => self.execute_next_instruction_debug::<CeInt16>(b2),
                        0b10 => self.execute_next_instruction_debug::<CeInt32>(b2),
                        0b11 => self.vm.execute_next_instruction(),
                        _ => unsafe { unreachable_unchecked() },
                    }
                }
                0b1010 => {
                    // Input
                    self.execute_next_instruction_debug::<CeInt32>(curr_instruction_byte);
                }
                0b1011 => {
                    // Output
                    self.vm.execute_next_instruction();
                }
                _ => unsafe { unreachable_unchecked() },
            }
        }
    }

    fn execute_next_instruction_and_debug_jmp(&mut self) {
        self.vm.execute_next_instruction();

        // instruction ptr should point to the next instruction; if the vm's instruction ptr is
        // different, it has executed a jump
        if self.vm.instruction_ptr != self.instruction_ptr {
            // Notify that we jumped
            println!("<Debug> JMP'ed to {}", self.vm.instruction_ptr);
        }
    }

    fn execute_next_instruction_debug<T: EndianConversion + 'static>(
        &mut self,
        target_location_bits: u8,
    ) {
        let target_location_type = get_location_info(target_location_bits);
        let target_data_type = TypeId::of::<T>();

        self.vm.execute_next_instruction();

        // TODO
        let register_name: String = match target_location_type {
            Location::Memory(n) | Location::Register(n) => match n {
                0 => "sp".to_owned(),
                x => format!("r{}", x),
            },
        };

        println!(
            "<Debug> did something to {}",
            match target_location_type {
                Location::Memory(..) => format!("memory at value of {}", register_name),
                Location::Register(..) => format!("{}", register_name),
            }
        );
    }

    fn cmp_instr<T: EndianConversion + PartialOrd + From<i8> + 'static>(&mut self, _: u8, b3: u8) {
        self.execute_next_instruction_debug::<T>(b3 >> 4);
    }

    fn lod_instr<T: EndianConversion + 'static>(&mut self, loc: u8, _: T) {
        self.execute_next_instruction_debug::<T>(loc);
    }

    pub(crate) fn is_done(&self) -> bool {
        self.vm.is_done()
    }
}

enum Location {
    Memory(u8),
    Register(u8),
}

fn get_location_info(bits: u8) -> Location {
    if (bits & 0b1000) != 0 {
        Location::Memory(bits & 0b0111)
    } else {
        Location::Register(bits & 0b0111)
    }
}
