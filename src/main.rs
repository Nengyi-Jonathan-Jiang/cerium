pub use crate::cerium::assembler::CeriumAssembler;
use crate::cerium::cerium_error::{BasicCeriumError, CeriumError};
pub use crate::cerium::vm::CeriumVM;
use crate::cerium::vm::{config_growable_memory_max_size, DebugCeriumVM};
use crate::util::ansi::colors::{red, reset};
use crate::util::ansi::enable_ansi;
use std::env::{args};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::{iter, panic};
use std::iter::{Peekable};

mod cerium;
mod util;

fn main() {
    enable_ansi();
    setup_panic_handler();

    let mut args = args().skip(1).peekable();

    match args.next() {
        None => help(),
        Some(first_arg) => match first_arg.as_str() {
            "assemble" => {
                let (files, mut output_file) = open_input_files_and_output_file(&mut args);
                let assembled_program = assemble(files);

                output_file.write(&*assembled_program).unwrap_or_else(|e| {
                    BasicCeriumError::throw_string(e.to_string())
                });
            }
            "debug-asm" => {
                handle_max_memory_arg_if_exists(&mut args);

                let files = open_input_files(&mut args);
                let assembled_program = assemble(files);

                DebugCeriumVM::execute_program(assembled_program.iter().cloned());
            }
            "run-asm" => {
                handle_max_memory_arg_if_exists(&mut args);

                let files = open_input_files(&mut args);
                let assembled_program = assemble(files);

                CeriumVM::execute_program(assembled_program.iter().cloned());
            }
            "debug" => {
                handle_max_memory_arg_if_exists(&mut args);

                let program = read_binary_file(&mut args);

                DebugCeriumVM::execute_program(program);
            }
            _ => {
                handle_max_memory_arg_if_exists(&mut args);

                let program = read_binary_file(&mut iter::once(first_arg));

                CeriumVM::execute_program(program);
            }
        },
    };
}

fn handle_max_memory_arg_if_exists(args: &mut Peekable<impl Iterator<Item=String>>) {
    if let Some(_) = args.next_if_eq("--max-memory") {
        let max_memory_size = args
            .next()
            .unwrap_or_else(|| BasicCeriumError::throw_str("Expected memory size"))
            .parse()
            .unwrap_or_else(|_| BasicCeriumError::throw_str("Invalid memory size"));

        unsafe {
            config_growable_memory_max_size(max_memory_size);
        }
    }
}

fn setup_panic_handler() {
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

fn open_file_read(input_path: &str) -> File {
    File::open(Path::new(input_path)).unwrap_or_else(|_| {
        BasicCeriumError::throw_string(format!("File not found: {}", input_path))
    })
}

fn open_file_write(output_path: &str) -> File {
    OpenOptions::new().write(true).create(true).open(Path::new(output_path)).unwrap_or_else(|_| {
        BasicCeriumError::throw_string(format!("File not found: {}", output_path))
    })
}

fn open_input_files(args: &mut impl Iterator<Item=String>) -> Vec<File> {
    let mut input_files = Vec::new();
    while let Some(file_name) = args.next() {
        input_files.push(open_file_read(&file_name));
    }
    if input_files.is_empty() {
        BasicCeriumError::throw_str("No input file(s) provided")
    }

    input_files
}

fn open_input_files_and_output_file(args: &mut impl Iterator<Item=String>) -> (Vec<File>, File) {
    let mut file_paths = args.collect::<Vec<_>>();
    let output_path = file_paths.pop()
        .unwrap_or_else(|| BasicCeriumError::throw_str("No output file provided"));

    if file_paths.is_empty() {
        BasicCeriumError::throw_str("No input files provided")
    }

    let input_files = open_input_files(&mut file_paths.into_iter());
    let output_file = open_file_write(&output_path);
    (input_files, output_file)
}

fn read_binary_file(args: &mut impl Iterator<Item=String>) -> Vec<u8> {
    let mut file = open_file_read(
        &args
            .next()
            .unwrap_or_else(|| BasicCeriumError::throw_str("No binary file provided")),
    );

    let mut program = Vec::new();
    file.read_to_end(&mut program).expect("Could not read file");
    program
}

fn assemble(files: impl IntoIterator<Item=File>) -> Box<[u8]> {
    let mut input_concatenated = String::new();

    for mut file in files {
        let mut input = String::new();
        file.read_to_string(&mut input)
            .unwrap_or_else(|_| BasicCeriumError::throw_str("Unable to read file"));

        input_concatenated += &input;
        input_concatenated += "\n";
    }

    CeriumAssembler::assemble_casm(input_concatenated.as_str())
}

fn help() {
    println!("CeriumVM Usage:");
    println!("  cerium assemble <input-files> <output-file> | Assembles one or more Cerium assembly files to a .ce file");
    println!(
        "  cerium run-asm <input-files>                | Runs one or more Cerium assembly files"
    );
    println!("  cerium debug-asm <input-files>              | Runs one or more Cerium assembly files in debug mode");
    println!("  cerium <input-file>                         | Runs a Cerium binary file");
    println!(
        "  cerium debug <input-file>                   | Runs a Cerium binary file in debug mode"
    );
}

trait CeriumVmLike: Sized + Default {
    fn load_program(&mut self, program: impl IntoIterator<Item=u8>);
    fn is_done(&self) -> bool;
    fn execute_next_instruction(&mut self);

    fn execute_program(program: impl IntoIterator<Item=u8>) {
        let program = program.into_iter().collect::<Vec<_>>();

        let mut vm = Self::default();

        vm.load_program(program);

        while !vm.is_done() {
            vm.execute_next_instruction();
        }
    }
}

impl CeriumVmLike for CeriumVM {
    fn load_program(&mut self, program: impl IntoIterator<Item=u8>) {
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
    fn load_program(&mut self, program: impl IntoIterator<Item=u8>) {
        self.load_program(program)
    }

    fn is_done(&self) -> bool {
        self.is_done()
    }

    fn execute_next_instruction(&mut self) {
        self.execute_next_instruction()
    }
}
