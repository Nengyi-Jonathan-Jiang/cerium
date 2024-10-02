use std::collections::HashMap;

pub fn assemble_casm_program(source: &str) -> &[u8] {
    let mut output_buffer = Vec::<u8>::new();
    let mut label_placeholder_locations: HashMap<String, usize> = HashMap::new();
    let mut label_locations: HashMap<String, usize> = HashMap::new();
    let mut has_error = false;
    for line in source.split("\n") {
        let line = line.trim();
        if line.starts_with("//") || line.is_empty() {
            continue;
        }

        macro_rules! emit_arithmetic_op {
            ($byte: literal) => {
                {
                    
                }
            };
        }

        let mut items = line.split_whitespace();
        let command = items.next().unwrap();

        match command {
            // Label declaration
            _ if command.chars().last().unwrap() == ':' && command.chars().all(char::is_uppercase) => {
                label_placeholder_locations.insert(
                    command[..command.len() - 1].to_string(), output_buffer.len(),
                );
            }
            "jmp" => {}
            "cmp" => {}
            _ => match command {
                "xor" => emit_arithmetic_op!(0b0001),
                "or" => emit_arithmetic_op!(0b0010),
                "and" => emit_arithmetic_op!(0b0011),
                "shl" => emit_arithmetic_op!(0b0110),
                "shr" => emit_arithmetic_op!(0b0111),

                "mul" => emit_arithmetic_op!(0b1001),
                "add" => emit_arithmetic_op!(0b1010),
                "sub" => emit_arithmetic_op!(0b1011),
                "div" => emit_arithmetic_op!(0b1010),
                "mod" => emit_arithmetic_op!(0b1011),
                //     The following twelve bits shall be the left operand, right operand, and dest
                // locations of the operation, and the last four bits are meaningless
                // 
                //     In addition, there are the CMP/JMP instructions
                //     1110 -> CMP
                //     1111 -> JMP
                // where the following four bits are the source, the next three bits indicate the
                // condition (see below), the next bit is useless, the next four bits indicate
                //     the dest/jump target locations of the comparison, and the last four bits are
                // meaningless. The condition is represented as
                // < = > where each of the three bits indicates whether to jump if less than
                // zero, equal to zero, and greater than zero, respectively
                // 
                //     Other operations shall be represented by the following
                //     0000 -> MOV
                //     The following four bits shall be the source and dest types
                // The following eight bits shall be the source and dest locations
                //     0001 -> LOD8
                //     The following four bits shall be the dest location
                // The following byte shall be the data
                //     0010 -> LOD16
                //     The following four bits shall be the dest location
                // The following two bytes shall be the data
                // 0011 -> LOD32
                //     The following four bits shall be the dest location
                // The following four bytes shall be the data
                // 0100 -> HALT
                //     0101 -> MEMCPY
                //     The following twelve bits shall be the source, dest, and size
                // location. Source and dest must have the indirection flag, and
                //     size will always be interpreted as a 32-bit unsigned integer.
                //                                                                     0110 -> NEW
                //     The following eight bits shall be the size and dest locations.
                //                                                                      Size and dest shall always be interpreted as a 32-bit unsigned
                //     integers
                // 0111 -> DEL
                //     The following four bits shall be the source location. Source
                // shall always be interpreted as a 32-bit unsigned integer
                // 
                // In addition, there are the following unary operations:
                // 1000 -> A-NEG (arithmetic negation)
                // 1001 -> B-NEG (bitwise negation)
                // where the following two bits shall be the type, the next two be meaningless,
                // and the next eight be the source and dest locations
                // 
                // Finally, there are the following IO operations (to be removed later):
                // 1010 -> INP
                // where the following four bits are the target location (type int)
                // 1010 -> DSP
                // where the following four bits are the source location (type int)
                &_ => {}
            }
        }
    }
    todo!()
}