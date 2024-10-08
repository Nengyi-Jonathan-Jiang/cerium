pub use crate::cerium::assembler::CasmAssembler;
pub use crate::cerium::vm::CeriumVM;
use std::env::args;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

mod util;
mod cerium;

fn main() {
    let mut args = args().skip(1);
    match args.next() {
        None => do_collatz(),
        Some(first_arg) => {
            match first_arg.as_str() {
                "assemble" => assemble(
                    args.next().expect("No input file provided").as_str(),
                    args.next().expect("No output file provided").as_str(),
                ),
                _ => execute_ce_binary(first_arg.as_str()),
            }
        }
    };
}

fn assemble(input_path: &str, output_path: &str) {
    let mut input_file = File::open(Path::new(input_path)).expect(
        &format!("File not found: {}", input_path)
    );
    let mut input_file_str: String = String::default();
    input_file.read_to_string(&mut input_file_str).expect("Unable to read input file");

    let result_bytes = CasmAssembler::assemble(input_file_str.as_str());

    let mut output_file = File::create(Path::new(output_path)).expect(
        &format!("File not found: {}", output_path)
    );

    output_file.write(&*result_bytes).expect("Unable to write to output file");
}

fn execute_ce_binary(path: &str) {
    let path = path;
    let mut file = File::open(Path::new(path)).expect(
        &format!("File not found: {}", path)
    );
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).expect(
        "Failed to read file into buffer"
    );

    let mut vm = CeriumVM::new();
    vm.load_program(buffer.as_slice());

    while !vm.is_done() {
        vm.execute_next_instruction();
    }
    println!("Done");
}

fn do_collatz() {
    let mut vm = CeriumVM::new();
    vm.load_program(vec![
        // Prepare jumps
        // 00: [r5] <- 58
        0b_0011_0101,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00111010,
        // 05: [r6] <- 16
        0b_0011_0110,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00010000,
        // 10: [r7] <- 69
        0b_0011_0111,
        0b00000000,
        0b00000000,
        0b00000000,
        0b01000101,

        // Take input
        // 15: [r1] <- input
        0b_1010_0001,

        // Display
        // 16: output <- [r1]
        0b_1011_0001,

        // Check base case
        // 17: [r3] <- 1
        0b0011_0011,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000001,
        // 22: [r2] <- [r1] - [r3]
        0b11_10_1011,
        0b0001_0011,
        0b0010_0000,
        // 25: JMP [r7] IF [r2] == 0
        0b11_10_1111,
        0b0010_010_0,
        0b0111_0000,
        // Check parity
        // 28: [r2] <- 2
        0b0011_0010,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000010,
        // 33: [r2] <- [r1] % [r2]
        0b11_10_1101,
        0b0001_0010,
        0b0010_0000,
        // 36: JMP [r5] IF [r2] == 0
        0b11_10_1111,
        0b0010_010_0,
        0b0101_0000,

        // Odd:
        // 39: [r2] <- 3
        0b0011_0010,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000011,
        // 44: [r1] <- [r1] * [r2]
        0b11_10_1001,
        0b0001_0010,
        0b0001_0000,
        // 47: [r2] <- 1
        0b0011_0010,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000001,
        // 52: [r1] <- [r1] + [r2]
        0b11_10_1010,
        0b0001_0010,
        0b0001_0000,
        // 55: JMP [r6]
        0b11_00_1111,
        0b0000_111_0,
        0b0110_0000,

        // Even:
        // 58: [r2] <- 1
        0b0011_0010,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000001,
        // 63: [r1] <- [r1] >> [r1]
        0b11_10_0111,
        0b0001_0010,
        0b0001_0000,
        // 66: JMP [r6]
        0b11_00_1111,
        0b0000_111_0,
        0b0110_0000,
        // 69: HALT
        0b01000000,
    ].as_slice());

    while !vm.is_done() {
        // println!("Executing instruction...");
        vm.execute_next_instruction();
    }
    println!("Done");
}
