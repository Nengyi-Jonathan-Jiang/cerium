// #![allow(arithmetic_overflow)]

use super::register::Register;
use super::{CeFloat, CeInt16, CeInt32, CeInt8, CeWord, Pointer, RAM};
use crate::cerium::cerium_error::{CeriumVMError};
use crate::cerium::instruction::casm_instruction_parts::{BinOp, Condition, Location, Type, UnOp};
use crate::cerium::instruction::{
    casm_instruction_parts, CASMInstruction, CASMInstructionSourceStream,
};
use crate::cerium::memory_buffer::{CeriumPrimitiveType, EndianConversion, MemoryBufferPtr};
use crate::match_cerium_type;
use std::hint::unreachable_unchecked;
use std::io;
use std::io::{IsTerminal, Write};
use std::mem::size_of;

#[derive(Default)]
pub struct CeriumVM {
    pub memory: RAM,
    pub registers: [Register; 8],
    pub instruction_ptr: CeWord,
    pub done: bool,
}

impl CASMInstructionSourceStream for CeriumVM {
    #[inline(always)]
    fn get_next<T: EndianConversion>(&mut self) -> T {
        self.get_next_instruction_and_inc_ip()
    }
}

impl CeriumVM {
    pub fn new() -> CeriumVM {
        Default::default()
    }

    pub fn load_program(&mut self, program: impl IntoIterator<Item = u8>) {
        let program_bytes = program.into_iter().collect::<Vec<_>>();
        let program_len = program_bytes.len() as CeInt32;

        unsafe { self.memory.write(Pointer::from(0), program_bytes) }

        // Set the instruction pointer to zero
        self.instruction_ptr = 0;
        // Set the stack pointer to point after the program bytes
        unsafe { self.registers[0].get().write(program_len) }
    }

    #[inline(always)]
    fn get_register<T: EndianConversion>(&mut self, register: casm_instruction_parts::Register) -> MemoryBufferPtr<T> {
        unsafe {
            self.registers
                .get_mut(register.to_bits() as usize)
                .unwrap_unchecked()
                .get()
        }
    }

    #[inline(always)]
    pub(crate) fn get_memory<T: EndianConversion>(&mut self, register: casm_instruction_parts::Register) -> MemoryBufferPtr<T> {
        let register_value = self.get_register::<CeInt32>(register).get() as CeWord;
        self.memory.at(Pointer::new(register_value))
    }

    #[inline(always)]
    pub(crate) fn get_location<T: EndianConversion>(&mut self, loc: Location) -> MemoryBufferPtr<T> {
        let register = loc.register();
        if loc.indirect() {
            self.get_memory(register)
        } else {
            self.get_register(register)
        }
    }

    #[inline(always)]
    pub(crate) fn get_word_for_location(&mut self, location: Location) -> CeWord {
        self.get_location::<CeInt32>(location).get() as CeWord
    }

    #[inline(always)]
    pub(crate) fn get_next_instruction_and_inc_ip<T: EndianConversion>(&mut self) -> T {
        let res: T = self.memory.at(Pointer::from(self.instruction_ptr)).get();
        self.instruction_ptr += size_of::<T>() as CeWord;
        res
    }

    pub fn execute_next_instruction(&mut self) {
        let next_instruction = CASMInstruction::parse_from_stream(self);

        match next_instruction {
            CASMInstruction::Mov {
                src_ty,
                dst_ty,
                src,
                dst,
            } => {
                fn _f<T: CeriumPrimitiveType>(
                    vm: &mut CeriumVM,
                    dst_ty: Type,
                    src: Location,
                    dst: Location,
                ) {
                    let val = vm.get_location::<T>(src).get();

                    fn _f<T: CeriumPrimitiveType, T2: CeriumPrimitiveType>(
                        vm: &mut CeriumVM,
                        dst: Location,
                        val: T,
                    ) {
                        unsafe {
                            vm.get_location::<T2>(dst)
                                .write(val.cast_to_primitive())
                        }
                    }
                    match_cerium_type!(match dst_ty => _f::<T>(vm, dst, val));
                }

                match_cerium_type!(match src_ty => _f(self, dst_ty, src, dst))
            }
            CASMInstruction::Const8(loc, dat) => self.mov_const(loc, dat),
            CASMInstruction::Const16(loc, dat) => self.mov_const(loc, dat),
            CASMInstruction::Const32(loc, dat) => self.mov_const(loc, dat),
            CASMInstruction::Halt => self.done = true,
            CASMInstruction::Memcpy { src, dst, size } => {
                let size = self.get_word_for_location(size).into();
                let src = self.get_word_for_location(src).into();
                let dest = self.get_word_for_location(dst).into();

                self.memory.memcpy(src, dest, size);
            }
            CASMInstruction::New { size, dst } => {
                let size = self.get_word_for_location(size);
                let res = CeWord::from(self.memory.allocate(size)) as CeInt32;

                unsafe {
                    self.get_location::<CeInt32>(dst).write(res);
                }
            }
            CASMInstruction::Del { src } => {
                let src = self.get_word_for_location(src).into();
                self.memory.deallocate(src);
            }
            CASMInstruction::Cmp { ty, src, dst, cnd } => {
                fn _f<T: CeriumPrimitiveType>(
                    vm: &mut CeriumVM,
                    src: Location,
                    dst: Location,
                    cnd: Condition,
                ) {
                    let val: T = vm.get_location(src).get();
                    let compare_result = cnd.test(val) as CeInt8;
                    unsafe {
                        vm.get_location(dst).write(compare_result);
                    }
                }

                match_cerium_type!(match ty => _f(self, src, dst, cnd))
            }
            CASMInstruction::Jmp { ty, src, tgt, cnd } => {
                fn _f<T: CeriumPrimitiveType>(
                    vm: &mut CeriumVM,
                    src: Location,
                    tgt: Location,
                    cnd: Condition,
                ) {
                    if cnd.test(vm.get_location::<T>(src).get()) {
                        vm.instruction_ptr = vm.get_word_for_location(tgt);
                    }
                }

                match_cerium_type!(match ty => _f(self, src, tgt, cnd))
            }
            CASMInstruction::BinOp {
                op,
                ty,
                src1,
                src2,
                dst,
            } => {
                fn _f<T: CeriumPrimitiveType>(
                    vm: &mut CeriumVM,
                    op: BinOp,
                    src1: Location,
                    src2: Location,
                    dst: Location,
                ) {
                    let operand_1: T = vm.get_location(src1).get();
                    let operand_2: T = vm.get_location(src2).get();
                    let res = op.apply(operand_1, operand_2);
                    unsafe { vm.get_location(dst).write(res) }
                }

                match_cerium_type!(match ty => _f(self, op, src1, src2, dst))
            }
            CASMInstruction::UnOp { op, ty, src, dst } => {
                fn _f<T: CeriumPrimitiveType>(
                    vm: &mut CeriumVM,
                    op: UnOp,
                    src: Location,
                    dst: Location,
                ) {
                    let operand: T = vm.get_location(src).get();
                    let res = op.apply(operand);
                    unsafe { vm.get_location(dst).write(res) }
                }

                match_cerium_type!(match ty => _f(self, op, src, dst))
            }
            CASMInstruction::Input(dst) => {
                let value: CeInt32;

                loop {
                    print!("<CeriumVM> Enter a number: ");
                    io::stdout().flush().unwrap();

                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap_or_else(|err| {
                        CeriumVMError::throw_string(format!("Failed to read input: {}", err))
                    });

                    if !io::stdout().is_terminal() || !io::stdin().is_terminal() {
                        print!("{}", input);
                    }

                    match input.trim().parse::<CeInt32>() {
                        Ok(v) => {
                            value = v;
                            break;
                        }
                        Err(err) => {
                            println!("<CeriumVM> Invalid integer input. {}", err);
                        }
                    }
                }

                unsafe {
                    self.get_location::<CeInt32>(dst).write(value);
                }
            }
            CASMInstruction::Output(src) => {
                println!(
                    "<CeriumVM> {}",
                    self.get_location::<CeInt32>(src).get()
                );
            }
            CASMInstruction::NoOp => {}

            // parse_next_instruction will never emit these instructions
            CASMInstruction::Data(_) => unsafe { unreachable_unchecked() },
            CASMInstruction::Label(_) => unsafe { unreachable_unchecked() },
            CASMInstruction::ConstLabel(..) => unsafe { unreachable_unchecked() },
        }
    }

    #[inline(always)]
    fn mov_const<T: EndianConversion>(&mut self, loc: Location, dat: T) {
        unsafe { self.get_location::<T>(loc).write(dat) }
    }

    pub fn is_done(&self) -> bool {
        self.done
    }
}
