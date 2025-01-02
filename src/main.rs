pub use crate::cerium::assembler::CeriumAssembler;
use crate::cerium::cerium_error::{BasicCeriumError, CeriumError};
pub use crate::cerium::vm::CeriumVM;
use crate::cerium::vm::DebugCeriumVM;
use crate::util::ansi::colors::{red, reset};
use crate::util::ansi::enable_ansi;
use std::env::args;
use std::fs::File;
use std::io::{Read, Write};
use std::panic;
use std::path::Path;

mod cerium;
mod util;

fn main() {
    enable_ansi();
    use_panic_handler();

    let mut args = args().skip(1);
    match args.next() {
        None => help(),
        Some(first_arg) => match first_arg.as_str() {
            "assemble" => assemble(
                args.next()
                    .unwrap_or_else(|| BasicCeriumError::throw_str("No input file provided"))
                    .as_str(),
                args.next()
                    .unwrap_or_else(|| BasicCeriumError::throw_str("No output file provided"))
                    .as_str(),
            ),
            "run-asm" => assemble_and_execute(
                args.next()
                    .unwrap_or_else(|| BasicCeriumError::throw_str("No input file provided"))
                    .as_str(),
            ),
            "debug-asm" => assemble_and_debug(
                args.next()
                    .unwrap_or_else(|| BasicCeriumError::throw_str("No input file provided"))
                    .as_str(),
            ),
            _ => execute_ce_binary(first_arg.as_str()),
        },
    };
}

fn use_panic_handler() {
    let default_panic = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let err = panic_info.payload();

        if let Some(err) = err.downcast_ref::<Box<dyn CeriumError>>() {
            println!("{}{}{}", red(), err.message(), reset());
        } else {
            default_panic(panic_info);
        }
    }));
}

fn assemble(input_path: &str, output_path: &str) {
    let mut input_file = open_file(input_path);
    let mut input_file_str: String = String::default();
    input_file
        .read_to_string(&mut input_file_str)
        .unwrap_or_else(|_| BasicCeriumError::throw_str("Unable to read input file"));

    let result_bytes = CeriumAssembler::assemble_casm(input_file_str.as_str());

    let mut output_file = open_file(output_path);

    output_file
        .write(&*result_bytes)
        .unwrap_or_else(|_| BasicCeriumError::throw_str("Unable to write to output file"));
}

fn open_file(input_path: &str) -> File {
    File::open(Path::new(input_path)).unwrap_or_else(|_| {
        BasicCeriumError::throw_string(format!("File not found: {}", input_path))
    })
}

fn assemble_and_execute(input_path: &str) {
    let mut input_file = open_file(input_path);
    let mut input_file_str: String = String::default();
    input_file
        .read_to_string(&mut input_file_str)
        .unwrap_or_else(|_| BasicCeriumError::throw_str("Unable to read input file"));

    let result_bytes = CeriumAssembler::assemble_casm(input_file_str.as_str());

    CeriumVM::execute_program(result_bytes.iter().cloned());

    println!("Done");
}

fn assemble_and_debug(input_path: &str) {
    let mut input_file = open_file(input_path);
    let mut input_file_str: String = String::default();
    input_file
        .read_to_string(&mut input_file_str)
        .unwrap_or_else(|_| BasicCeriumError::throw_str("Unable to read input file"));

    let result_bytes = CeriumAssembler::assemble_casm(input_file_str.as_str());

    DebugCeriumVM::execute_program(result_bytes.iter().cloned());

    println!("Done");
}

fn execute_ce_binary(path: &str) {
    let mut file = open_file(path);
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer)
        .unwrap_or_else(|_| BasicCeriumError::throw_str("Failed to read file into buffer"));

    CeriumVM::execute_program(buffer.iter().cloned());

    println!("Done");
}

fn help() {
    println!("CeriumVM Usage:");
    println!("  cerium assemble <input-file> <output-file> | Assembles a .casm file to a .ce file");
    println!("  cerium run-asm <input-file>                | Assembles and runs a .casm file");
    println!("  cerium <input-file>                        | Runs a .ce file");
    println!("  cerium debug-asm <input-file>              | Runs a .casm file and shows the state of the stack and registers while the program is executing");
}

trait CeriumVmLike: Sized + Default {
    fn load_program(&mut self, program: impl IntoIterator<Item = u8>);
    fn is_done(&self) -> bool;
    fn execute_next_instruction(&mut self);

    fn execute_program(program: impl IntoIterator<Item = u8>) {
        let program = program.into_iter().collect::<Vec<_>>();

        let mut vm = Self::default();

        vm.load_program(program);

        while !vm.is_done() {
            vm.execute_next_instruction();
        }
    }
}

impl CeriumVmLike for CeriumVM {
    fn load_program(&mut self, program: impl IntoIterator<Item = u8>) {
        self.load_program(program)
    }

    fn is_done(&self) -> bool {
        self.is_done()
    }

    fn execute_next_instruction(&mut self) {
        self.execute_next_instruction()
    }
}

impl CeriumVmLike for DebugCeriumVM {
    fn load_program(&mut self, program: impl IntoIterator<Item = u8>) {
        self.load_program(program)
    }

    fn is_done(&self) -> bool {
        self.is_done()
    }

    fn execute_next_instruction(&mut self) {
        self.execute_next_instruction()
    }
}
