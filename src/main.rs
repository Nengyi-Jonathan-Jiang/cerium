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
        None => help(),
        Some(first_arg) => {
            match first_arg.as_str() {
                "assemble" => assemble(
                    args.next().expect("No input file provided").as_str(),
                    args.next().expect("No output file provided").as_str(),
                ),
                "run-asm" => assemble_and_execute(
                    args.next().expect("No input file provided").as_str()
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

fn assemble_and_execute(input_path: &str) {
    let mut input_file = File::open(Path::new(input_path)).expect(
        &format!("File not found: {}", input_path)
    );
    let mut input_file_str: String = String::default();
    input_file.read_to_string(&mut input_file_str).expect("Unable to read input file");

    let result_bytes = CasmAssembler::assemble(input_file_str.as_str());
    
    let mut vm = CeriumVM::new();
    vm.load_program(result_bytes);

    while !vm.is_done() {
        vm.execute_next_instruction();
    }
    println!("Done");
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

fn help() {
    println!("CeriumVM Usage:");
    println!("  cerium assemble <input-file> <output-file> | Assembles a .casm file to a .ce file");
    println!("  cerium run-asm <input-file>                | Assembles and runs a .casm file");
    println!("  cerium <input-file>                        | Runs a .ce file");
}
