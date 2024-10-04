// #![allow(arithmetic_overflow)]

use std::fmt::LowerHex;
use super::register::CeriumRegister;
use super::{CeFloat, CeInt16, CeInt32, CeInt8, CeWord, CeriumPtr, CeriumRAM};
use std::ops::*;
use text_io::read;
use crate::cerium::memory_buffer::{Endianness, MemoryBuffer, MemoryBufferPtr};

#[derive(Default)]
pub struct CeriumVM {
    memory: CeriumRAM,
    registers: [CeriumRegister; 8],
    instruction_ptr: CeWord,
    program: MemoryBuffer,
    done: bool,
}

impl CeriumVM {
    pub fn new() -> CeriumVM { Default::default() }

    pub fn load_program(&mut self, program: impl Into<MemoryBuffer>) {
        self.program = program.into();
        self.instruction_ptr = 0;
    }

    fn get_register<T: Endianness>(&mut self, bits: u8) -> MemoryBufferPtr<T> {
        self.registers.get_mut((bits & 0b111) as usize).unwrap().get()
    }

    fn get_memory<T: Endianness>(&mut self, bits: u8) -> MemoryBufferPtr<T> {
        let register_value = self.get_register::<CeInt32>(bits).get() as CeWord;
        self.memory.at(CeriumPtr::new(register_value)).unwrap()
    }

    fn get_location<T: Endianness>(&mut self, bits: u8) -> MemoryBufferPtr<T> {
        if (bits & 0b1000) != 0 {
            self.get_memory(bits)
        } else {
            self.get_register(bits)
        }
    }

    fn get_word_for_location(&mut self, bits: u8) -> CeWord {
        self.get_location::<CeInt32>(bits).get() as CeWord
    }

    fn get_next_and_inc_ip<T: Endianness>(&mut self) -> T {
        let res = self.program.get::<T>(self.instruction_ptr as usize).get();
        self.instruction_ptr += size_of::<T>() as CeWord;
        res
    }

    pub fn execute_next_instruction(&mut self) {
        // print!("executing instruction at byte index {:3}; ", self.instruction_ptr as usize);
        // println!("Registers: {:08x}, {:08x}, {:08x}, {:08x}, {:08x}, {:08x}, {:08x}, {:08x}", 
        //          self.registers[0].get::<CeInt32>().get(),
        //          self.registers[1].get::<CeInt32>().get(),
        //          self.registers[2].get::<CeInt32>().get(),
        //          self.registers[3].get::<CeInt32>().get(),
        //          self.registers[4].get::<CeInt32>().get(),
        //          self.registers[5].get::<CeInt32>().get(),
        //          self.registers[6].get::<CeInt32>().get(),
        //          self.registers[7].get::<CeInt32>().get()
        // );

        let curr_instruction_byte = self.get_next_and_inc_ip::<u8>();

        if (curr_instruction_byte >> 6) == 3 {
            // Ternary instructions
            let instruction_part = curr_instruction_byte & 0b00001111;
            let type_part = (curr_instruction_byte & 0b00110000) >> 4;

            let b2 = self.get_next_and_inc_ip::<u8>();
            let b3 = self.get_next_and_inc_ip::<u8>();

            macro_rules! do_instruction {
                ($op: expr; $reason: expr) => {
                    match type_part {
                        0b00 => self.do_binop::<CeInt8>(b2, b3, $op),
                        0b01 => self.do_binop::<CeInt16>(b2, b3, $op),
                        0b10 => self.do_binop::<CeInt32>(b2, b3, $op),
                        0b11 => panic!($reason),
                        _ => unreachable!()
                    }
                };
                ($op: expr) => {
                    match type_part {
                        0b00 => self.do_binop::<CeInt8>(b2, b3, $op),
                        0b01 => self.do_binop::<CeInt16>(b2, b3, $op),
                        0b10 => self.do_binop::<CeInt32>(b2, b3, $op),
                        0b11 => self.do_binop::<CeFloat>(b2, b3, $op),
                        _ => unreachable!()
                    }
                };
            }

            match instruction_part {
                0b0000 => (), // NO-OP
                0b0001 => do_instruction!(BitXor::bitxor; "Cannot apply XOR to float"),
                0b0010 => do_instruction!(BitOr::bitor; "Cannot apply OR to float"),
                0b0011 => do_instruction!(BitAnd::bitand; "Cannot apply AND to float"),
                0b0100 | 0b0101 => (), // NO-OP
                0b0110 => do_instruction!(Shl::shl; "Cannot apply SHL to float"),
                0b0111 => do_instruction!(Shr::shr; "Cannot apply SHR to float"),
                0b1000 => (), // NO-OP
                0b1001 => do_instruction!(Mul::mul),
                0b1010 => do_instruction!(Add::add),
                0b1011 => do_instruction!(Sub::sub),
                0b1100 => do_instruction!(Div::div),
                0b1101 => do_instruction!(modulo),
                0b1110 => match type_part { // CMP
                    0b00 => self.cmp_instr::<CeInt8>(b2, b3),
                    0b01 => self.cmp_instr::<CeInt16>(b2, b3),
                    0b10 => self.cmp_instr::<CeInt32>(b2, b3),
                    0b11 => self.cmp_instr::<CeFloat>(b2, b3),
                    _ => unreachable!()
                }
                0b1111 => match type_part { // JMP
                    0b00 => self.jmp_instr::<CeInt8>(b2, b3),
                    0b01 => self.jmp_instr::<CeInt16>(b2, b3),
                    0b10 => self.jmp_instr::<CeInt32>(b2, b3),
                    0b11 => self.jmp_instr::<CeFloat>(b2, b3),
                    _ => unreachable!()
                }
                _ => unreachable!()
            }
        } else {
            let instruction_part = curr_instruction_byte >> 4;
            match instruction_part {
                0b0000 => { // MOV
                    let b2 = self.get_next_and_inc_ip::<u8>();
                    let t1 = (curr_instruction_byte >> 2) & 3;
                    let t2 = curr_instruction_byte & 3;

                    macro_rules! mov_match_case {
                    (type = $t: ty, $b2: ident, $t2: ident) => {
                        unsafe {
                            let val = self.get_location::<$t>($b2 >> 4).get();
                            match $t2 {
                                0b00 => self.get_location::<CeInt8>($b2).write((val as CeInt8)),
                                0b01 => self.get_location::<CeInt16>($b2).write((val as CeInt16)),
                                0b10 => self.get_location::<CeInt32>($b2).write((val as CeInt32)),
                                0b11 => self.get_location::<CeFloat>($b2).write((val as CeFloat)),
                                _ => unreachable!()
                            }
                        }
                    };
                }

                    match t1 {
                        0b00 => mov_match_case!(type = CeInt8, t2, b2),
                        0b01 => mov_match_case!(type = CeInt16, t2, b2),
                        0b10 => mov_match_case!(type = CeInt32, t2, b2),
                        0b11 => mov_match_case!(type = CeFloat, t2, b2),
                        _ => unreachable!()
                    }
                }
                0b0001 => { // LOD8
                    let dat = self.get_next_and_inc_ip::<CeInt8>();
                    self.lod_instr(curr_instruction_byte, dat);
                }
                0b0010 => { // LOD16
                    let dat = self.get_next_and_inc_ip::<CeInt16>();
                    self.lod_instr(curr_instruction_byte, dat);
                }
                0b0011 => { // LOD32
                    let dat = self.get_next_and_inc_ip::<CeInt32>();
                    self.lod_instr(curr_instruction_byte, dat);
                }
                0b0100 => {
                    self.done = true;
                } // NOOP
                0b0101 => {
                    // MEMCPY
                    let b2 = self.get_next_and_inc_ip::<u8>();

                    let size = self.get_register::<CeInt32>(curr_instruction_byte).get() as CeWord;
                    let src = self.get_register::<CeInt32>(b2 >> 4).get() as CeWord;
                    let dest = self.get_register::<CeInt32>(b2).get() as CeWord;

                    self.memory.memcpy(src.into(), dest.into(), size.into()).unwrap();
                }
                0b0110 => { // NEW
                    let b2 = self.get_next_and_inc_ip::<u8>();
                    let size = self.get_register::<CeInt32>(b2 >> 4).get() as CeWord;
                    let res = CeWord::from(self.memory.allocate(size).unwrap());

                    unsafe {
                        self.get_register::<CeInt32>(b2).write(res as CeInt32);
                    }
                }
                0b0111 => { // DEL
                    let b2 = self.get_next_and_inc_ip::<u8>();
                    let src = self.get_register::<CeInt32>(b2 >> 4).get() as CeWord;
                    self.memory.deallocate(src.into()).unwrap();
                }
                0b1000 => { // NEG
                    let type_part = (curr_instruction_byte >> 2) & 0b11;
                    let b2 = self.get_next_and_inc_ip::<u8>();

                    match type_part {
                        0b00 => self.do_unop::<CeInt8>(b2, Neg::neg),
                        0b01 => self.do_unop::<CeInt16>(b2, Neg::neg),
                        0b10 => self.do_unop::<CeInt32>(b2, Neg::neg),
                        0b11 => self.do_unop::<CeFloat>(b2, Neg::neg),
                        _ => unreachable!()
                    }
                }
                0b1001 => {
                    // Bitwise negation
                    let type_part = (curr_instruction_byte >> 2) & 0b11;
                    let b2 = self.get_next_and_inc_ip::<u8>();

                    match type_part {
                        0b00 => self.do_unop::<CeInt8>(b2, Not::not),
                        0b01 => self.do_unop::<CeInt16>(b2, Not::not),
                        0b10 => self.do_unop::<CeInt32>(b2, Not::not),
                        0b11 => panic!("Cannot apply bitwise negation to float"),
                        _ => unreachable!()
                    }
                }
                0b1010 => {
                    print!("<CeriumVM> Enter a number: ");
                    unsafe {
                        self.get_register::<CeInt32>(curr_instruction_byte).write(read!());
                    }
                }
                0b1011 => {
                    println!("{}", self.get_register::<CeInt32>(curr_instruction_byte).get());
                }
                _ => unreachable!()
            }
        }
    }

    fn test_condition<T: Endianness + PartialOrd + From<i8>>(&mut self, b2: u8) -> bool {
        let src: T = self.get_location::<T>(b2 >> 4).get();
        match b2 & 0b1110 {
            0b0000 => false,
            0b0010 => src > T::from(0),
            0b0100 => src == T::from(0),
            0b0110 => src >= T::from(0),
            0b1000 => src < T::from(0),
            0b1010 => src != T::from(0),
            0b1100 => src <= T::from(0),
            0b1110 => true,
            _ => unreachable!(),
        }
    }

    fn do_binop<T: Endianness>(&mut self, b2: u8, b3: u8, op: fn(T, T) -> T) {
        let val1 = self.get_location::<T>(b2 >> 4).get();
        let val2 = self.get_location::<T>(b2).get();
        let res = op(val1, val2);
        unsafe {
            self.get_location::<T>(b3 >> 4).write(res);
        }
    }

    fn do_unop<T: Endianness>(&mut self, b2: u8, op: fn(T) -> T) {
        let src = self.get_location::<T>(b2 >> 4).get();
        let mut dst = self.get_location::<T>(b2);
        unsafe { dst.write(op(src)) }
    }

    fn jmp_instr<T: Endianness + PartialOrd + From<i8>>(&mut self, b2: u8, b3: u8) {
        if self.test_condition::<T>(b2) {
            self.instruction_ptr = self.get_word_for_location(b3 >> 4);
        }
    }

    fn cmp_instr<T: Endianness + PartialOrd + From<i8>>(&mut self, b2: u8, b3: u8) {
        let result = self.test_condition::<T>(b2) as i8;
        unsafe { self.get_location::<CeInt8>(b3 >> 4).write(result); }
    }

    fn lod_instr<T: Endianness>(&mut self, loc: u8, dat: T) {
        unsafe { self.get_location::<T>(loc).write(dat) }
    }

    pub fn is_done(&self) -> bool { self.done }
}

fn modulo<T: Add<Output = T> + Rem<Output = T> + Copy>(x: T, m: T) -> T {
    (x % m + m) % m
}