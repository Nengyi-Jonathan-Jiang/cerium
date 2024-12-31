// #![allow(arithmetic_overflow)]

use super::super::instruction::casm_instruction_parts;
use super::{CeFloat, CeInt16, CeInt32, CeInt8, CeWord};
use crate::cerium::instruction::casm_instruction_parts::{Location, Register, Type};
use crate::cerium::instruction::{CASMInstruction, CASMInstructionSourceStream};
use crate::cerium::memory_buffer::{CeriumPrimitiveType, EndianConversion};
use crate::util::ansi::colors::yellow;
use crate::util::ansi::colors::{blue, cyan, default, green, purple, reset};
use crate::{match_cerium_type, CeriumVM};
use ansi_width::ansi_width;
use std::cmp::max;
use std::collections::HashMap;
use std::hint::unreachable_unchecked;
use std::mem::size_of;
use std::ops::AddAssign;

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
        let old_vm_ip = self.vm.instruction_ptr;
        self.vm.instruction_ptr = self.instruction_ptr;
        let res = self.vm.get_next_instruction_and_inc_ip();
        self.instruction_ptr = self.vm.instruction_ptr;
        self.vm.instruction_ptr = old_vm_ip;

        res
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

    fn sync_instruction_pointer_with_vm(&mut self) {
        self.instruction_ptr = self.vm.instruction_ptr;
    }

    #[inline(always)]
    fn get_word_for_location(&mut self, bits: u8) -> CeWord {
        self.vm.get_location::<CeInt32>(bits).get() as CeWord
    }

    pub fn execute_next_instruction(&mut self) {
        self.sync_instruction_pointer_with_vm();

        let next_instruction = CASMInstruction::parse_from_stream(self);

        // Hacky way to get around input being weird
        if let CASMInstruction::Input(..) = next_instruction {
        } else {
            Self::pretty_print_instruction(&next_instruction)
        }

        match next_instruction {
            CASMInstruction::Mov { dst_ty, dst, .. } => {
                self.vm.execute_next_instruction();
                self.debug(dst_ty, dst)
            }
            CASMInstruction::Lod8(loc, _) => {
                self.vm.execute_next_instruction();
                self.debug(Type::Int8, loc)
            }
            CASMInstruction::Lod16(loc, _) => {
                self.vm.execute_next_instruction();
                self.debug(Type::Int16, loc)
            }
            CASMInstruction::Lod32(loc, _) => {
                self.vm.execute_next_instruction();
                self.debug(Type::Int32, loc)
            }
            CASMInstruction::Halt => {
                println!();
                self.vm.execute_next_instruction();
            },
            CASMInstruction::Memcpy { src, dst, size } => {
                let size = self.get_word_for_location(size.to_bits());
                let src = self.get_word_for_location(src.to_bits());
                let dest = self.get_word_for_location(dst.to_bits());

                self.vm.execute_next_instruction();

                println!(
                    "{}MEMCPY'ed {} bytes from ptr {} to ptr {}{}",
                    yellow(),
                    size,
                    src,
                    dest,
                    reset(),
                );
            }
            CASMInstruction::New { size, dst } => {
                let size = self.get_word_for_location(size.to_bits());

                println!("{}Allocated {} bytes of memory{}", yellow(), size, reset(),);

                self.vm.execute_next_instruction();
                self.debug(Type::Int32, dst);
            }
            CASMInstruction::Del { src } => {
                let src = self.get_word_for_location(src.to_bits()).into();

                let size = self.vm.memory.get_allocation_size(src);
                self.vm.execute_next_instruction();

                println!(
                    "{}Deallocated {} bytes of memory at {}{}",
                    yellow(),
                    CeWord::from(size.unwrap()),
                    CeWord::from(src),
                    reset(),
                );
            }
            CASMInstruction::Jmp { .. } => {
                self.vm.execute_next_instruction();
                self.debug_jmp();
            }
            CASMInstruction::Cmp { ty, dst, .. }
            | CASMInstruction::BinOp { ty, dst, .. }
            | CASMInstruction::UnOp { ty, dst, .. } => {
                self.vm.execute_next_instruction();
                self.debug(ty, dst);
            }
            CASMInstruction::Input(dst) => {
                self.vm.execute_next_instruction();
                Self::pretty_print_instruction(&next_instruction);
                self.debug(Type::Int32, dst);
            }
            CASMInstruction::Output(_) => {
                println!();
                self.vm.execute_next_instruction();
            }
            CASMInstruction::NoOp => self.vm.execute_next_instruction(),

            // parse_next_instruction will never emit these instructions
            CASMInstruction::Data(_) => unsafe { unreachable_unchecked() },
            CASMInstruction::Label(_) => unsafe { unreachable_unchecked() },
            CASMInstruction::LodLabel(..) => unsafe { unreachable_unchecked() },
        }
    }

    fn pretty_print_instruction(instruction: &CASMInstruction) {
        let next_instruction_str = format!("{:?}", instruction);
        print!(
            "{}{} ",
            next_instruction_str,
            " ".repeat(max(30 - (ansi_width(&next_instruction_str) as i32), 0) as usize)
        );
    }

    fn debug_jmp(&mut self) {
        // instruction ptr should point to the next instruction; if the vm's instruction ptr is
        // different, it has executed a jump
        if self.vm.instruction_ptr != self.instruction_ptr {
            // Notify that we jumped
            println!(
                "{}jumped:{}    => {}{}{}",
                yellow(),
                reset(),
                purple(),
                self.vm.instruction_ptr,
                reset(),
            );
        }
        else {
            println!()
        }
    }

    fn debug(&mut self, dst_ty: Type, dst_loc: Location) {
        // Update the data type at dst_loc
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

        if dst_loc.indirect {
            // Print the stack, minus the program memory
            print!("{}memory:    {}(program * {} bytes){}, ", yellow(), blue(), self.program_length, reset());
            
            let mut curr_index = self.program_length as CeWord;
            let max_index = self.vm.memory.stack_capacity();
            let mut accum_empty: usize = 0;
            while curr_index < max_index {
                if let Some(ty) = self.memory_types.get(&curr_index) {
                    if accum_empty != 0 {
                        print!("{}(empty * {} bytes){}, ", blue(), accum_empty, reset());
                    }
                    
                    fn _f<T: CeriumPrimitiveType>(
                        vm: &mut CeriumVM,
                        ty: Type,
                        curr_index: &mut CeWord,
                    ) {
                        print!("{}{:?} {}{}{}, ", cyan(), ty, purple(), format!("{}", vm.memory.at::<T>((*curr_index).into()).unwrap().get()), reset());

                        curr_index.add_assign(size_of::<T>() as CeWord);
                    }

                    match_cerium_type!(match ty => _f(&mut self.vm, *ty, &mut curr_index));
                    
                    accum_empty = 0;
                } else {
                    curr_index += 1;
                    accum_empty += 1;
                }
            }
            println!();
        } else {
            print!("{}registers:{} |", yellow(), reset());
            for r in 0..8 {
                let is_changed_register = r as usize == dst_loc.register.to_bits() as usize;

                fn _f<T: CeriumPrimitiveType>(
                    vm: &mut CeriumVM,
                    register: Register,
                    is_changed_register: bool,
                ) {
                    print!(
                        "{:^31}|",
                        format!(
                            "{}{}{:?}{} = {}{}{}{}",
                            if is_changed_register { "[" } else { " " },
                            if is_changed_register { green() } else { default() },
                            register,
                            default(),
                            purple(),
                            vm.registers[register.to_bits() as usize].get::<T>().get(),
                            reset(),
                            if is_changed_register { "]" } else { " " },
                        ),
                    );
                }

                match_cerium_type!(match self.registers_types[r as usize] => _f(&mut self.vm, Register::from_bits(r).unwrap(), is_changed_register));
            }
            println!();
        }
    }

    pub fn is_done(&self) -> bool {
        self.vm.is_done()
    }
}
