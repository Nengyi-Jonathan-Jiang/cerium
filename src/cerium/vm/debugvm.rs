// #![allow(arithmetic_overflow)]

use super::{CeInt8, CeInt16, CeInt32, CeFloat, CeWord, Pointer};
use crate::cerium::instruction::casm_instruction_parts::{Location, Register, Type};
use crate::cerium::instruction::{CASMInstruction, CASMInstructionSourceStream};
use crate::cerium::memory_buffer::{CeriumPrimitiveType, EndianConversion};
use crate::{match_cerium_type, CeriumVM};
use std::collections::HashMap;
use std::hint::unreachable_unchecked;
use std::mem::size_of;
use super::super::instruction::casm_instruction_parts;

#[derive(Default)]
pub struct DebugCeriumVM {
    vm: CeriumVM,
    program_length: usize,
    instruction_ptr: CeWord,
    registers_types: [Type; 8],
    memory_types: HashMap<CeWord, Type>,
}

impl CASMInstructionSourceStream for DebugCeriumVM {
    fn get_next<T: EndianConversion>(&mut self) -> T {
        self.get_next_and_inc_ip()
    }
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
    fn get_word_for_location(&mut self, bits: u8) -> CeWord {
        self.vm.get_location::<CeInt32>(bits).get() as CeWord
    }

    pub fn execute_next_instruction(&mut self) {
        self.re_sync_ip_with_vm();

        let next_instruction = CASMInstruction::parse_from_stream(self);

        println!("{:?}", next_instruction);

        match CASMInstruction::parse_from_stream(self) {
            CASMInstruction::Mov { dst_ty, dst, .. } => {
                self.execute_next_instruction_debug(dst_ty, dst)
            }
            CASMInstruction::Lod8(loc, _) => self.execute_next_instruction_debug(Type::Int8, loc),
            CASMInstruction::Lod16(loc, _) => self.execute_next_instruction_debug(Type::Int16, loc),
            CASMInstruction::Lod32(loc, _) => self.execute_next_instruction_debug(Type::Int32, loc),
            CASMInstruction::Halt => self.vm.execute_next_instruction(),
            CASMInstruction::Memcpy { src, dst, size } => {
                let size = self.get_word_for_location(size.to_bits());
                let src = self.get_word_for_location(src.to_bits());
                let dest = self.get_word_for_location(dst.to_bits());

                self.vm.execute_next_instruction();

                println!(
                    "<Debug> MEMCPY'ed {} bytes from ptr {} to ptr {}",
                    size, src, dest
                );
            }
            CASMInstruction::New { size, dst } => {
                let size = self.get_word_for_location(size.to_bits());

                println!("<Debug> Allocated {} bytes of memory", size);
                self.execute_next_instruction_debug(Type::Int32, dst);
            }
            CASMInstruction::Del { src } => {
                let src = self.get_word_for_location(src.to_bits()).into();

                let size = self.vm.memory.get_allocation_size(src);
                self.vm.execute_next_instruction();

                println!(
                    "<Debug> Deallocated {} bytes of memory at {}",
                    CeWord::from(size.unwrap()),
                    CeWord::from(src)
                );
            }
            CASMInstruction::Jmp { .. } => self.execute_next_instruction_debug_jmp(),
            CASMInstruction::Cmp { ty, dst, .. }
            | CASMInstruction::BinOp { ty, dst, .. }
            | CASMInstruction::UnOp { ty, dst, .. } => {
                self.execute_next_instruction_debug(ty, dst);
            }
            CASMInstruction::Input(dst) => {
                self.execute_next_instruction_debug(Type::Int32, dst);
            }
            CASMInstruction::Output(_) => {
                self.vm.execute_next_instruction();
            }
            CASMInstruction::NoOp => self.vm.execute_next_instruction(),

            // parse_next_instruction will never emit these instructions
            CASMInstruction::Data(_) => unsafe { unreachable_unchecked() },
            CASMInstruction::Label(_) => unsafe { unreachable_unchecked() },
            CASMInstruction::LodLabel(..) => unsafe { unreachable_unchecked() },
        }
    }

    fn execute_next_instruction_debug_jmp(&mut self) {
        self.vm.execute_next_instruction();

        // instruction ptr should point to the next instruction; if the vm's instruction ptr is
        // different, it has executed a jump
        if self.vm.instruction_ptr != self.instruction_ptr {
            // Notify that we jumped
            println!("<Debug> JMP'ed to {}", self.vm.instruction_ptr);
        }
    }

    fn execute_next_instruction_debug(&mut self, dst_ty: Type, dst_loc: Location) {
        self.vm.execute_next_instruction();
        if dst_loc.indirect {
            let mem_loc = self.get_word_for_location(
                Location {
                    register: dst_loc.register,
                    indirect: false,
                }
                .to_bits(),
            );
            self.memory_types.insert(mem_loc, dst_ty);
        } else {
            self.registers_types[dst_loc.register.to_bits() as usize] = dst_ty;
        }

        print!("<Debug> ");
        if dst_loc.indirect {
            println!("did something to memory at value of {:?}", dst_loc.register);
        } else {
            print!("Registers are now:    ");
            for r in 0..8 {
                fn _f<T: CeriumPrimitiveType>(vm: &mut CeriumVM, register: Register) {
                    print!(
                        "{:?}={:<10} ",
                        register,
                        vm.registers[register.to_bits() as usize]
                            .get::<T>()
                            .get()
                    );
                }

                match_cerium_type!(match self.registers_types[r as usize] => _f(&mut self.vm, Register::from_bits(r).unwrap()));
            }
            println!();
        }
    }

    pub fn is_done(&self) -> bool {
        self.vm.is_done()
    }
}
