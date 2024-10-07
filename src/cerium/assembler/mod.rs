use std::collections::HashMap;
use crate::try_do;

pub struct CasmAssembler {
    output_buffer: Vec<u8>,
    label_placeholder_locations: Vec<(usize, String)>,
    label_locations: HashMap<String, usize>,
}

impl CasmAssembler {
    pub fn assemble(source: &str) -> Box<[u8]> {
        let mut assembler = CasmAssembler {
            output_buffer: vec![],
            label_placeholder_locations: Default::default(),
            label_locations: Default::default(),
        };

        for line in source.split("\n") {
            let line = line.trim();
            if line.starts_with("//") || line.is_empty() {
                continue;
            }

            if let None = assembler.parse_line(line.split_whitespace()) {
                println!("Invalid line: {}", line)
            }
        }

        assembler.insert_labels();

        assembler.output_buffer.into_boxed_slice()
    }

    fn parse_line<'a>(&mut self, mut items: impl Iterator<Item = &'a str>) -> Option<()> {
        let command = items.next().unwrap();

        match command {
            // Labels
            _ if command.chars().last().unwrap() == ':' && command.chars().rev().skip(1).all(
                Self::is_label_character
            ) => {
                self.set_label_value(command);
            }

            // Arithmetic operations
            "xor" => try_do!(self.parse_and_emit_binop(&mut items, 0b0001)),
            "or" => try_do!(self.parse_and_emit_binop(&mut items,  0b0010)),
            "and" => try_do!(self.parse_and_emit_binop(&mut items, 0b0011)),
            "shl" => try_do!(self.parse_and_emit_binop(&mut items, 0b0110)),
            "shr" => try_do!(self.parse_and_emit_binop(&mut items, 0b0111)),
            "mul" => try_do!(self.parse_and_emit_binop(&mut items, 0b1001)),
            "add" => try_do!(self.parse_and_emit_binop(&mut items, 0b1010)),
            "sub" => try_do!(self.parse_and_emit_binop(&mut items, 0b1011)),
            "div" => try_do!(self.parse_and_emit_binop(&mut items, 0b1100)),
            "mod" => try_do!(self.parse_and_emit_binop(&mut items, 0b1101)),

            // Other operations
            "jmp" => {
                let target_location = try_do!(Self::parse_location(items.next()));
                match try_do!(items.next()) {
                    "always" => {
                        self.output_buffer.push(0b11_00_1111_u8);
                        self.output_buffer.push(0b0000_111_0_u8);
                        self.output_buffer.push(target_location << 4);
                    }
                    "if" => {
                        let source_type = try_do!(Self::parse_type(items.next()));
                        let source_location = try_do!(Self::parse_location(items.next()));
                        let comparison_condition = try_do!(Self::parse_condition(items.next()));
                        self.output_buffer.push(0b11_00_1111_u8 | (source_type << 4));
                        self.output_buffer.push((source_location << 4) | comparison_condition);
                        self.output_buffer.push(target_location << 4);
                    }
                    _ => return None
                }
            }
            "cmp" => {
                let dest_location = try_do!(Self::parse_location(items.next()));

                try_do!(items.next());

                let source_type = try_do!(Self::parse_type(items.next()));
                let source_location = try_do!(Self::parse_location(items.next()));
                let comparison_condition = try_do!(Self::parse_condition(items.next()));
                self.output_buffer.push(0b11_00_1110_u8 | (source_type << 4));
                self.output_buffer.push((source_location << 4) | comparison_condition);
                self.output_buffer.push(dest_location << 4);
            }
            "mov" => {
                let dst_type = try_do!(Self::parse_type(items.next()));
                let dst_loc = try_do!(Self::parse_location(items.next()));
                try_do!(items.next());
                let src_type = try_do!(Self::parse_type(items.next()));
                let src_loc = try_do!(Self::parse_location(items.next()));

                self.output_buffer.push((src_type << 2) | dst_type);
                self.output_buffer.push((src_loc << 4) | dst_loc);
            }
            "lod" => {
                let dest = try_do!(Self::parse_location(items.next()));
                try_do!(items.next());
                match try_do!(items.next()) {
                    "b" => {
                        let value: u32 = try_do!(self.parse_integral_value(items.next()));
                        if (value & 0xffffff00) != 0 && (value & 0xffffff00) != 0xffffff00 {
                            return None;
                        }

                        self.output_buffer.push(0b00010000 | dest);
                        self.output_buffer.push(value as u8);
                    }
                    "s" => {
                        let value: u32 = try_do!(self.parse_integral_value(items.next()));
                        if (value & 0xffff0000) != 0 && (value & 0xffff0000) != 0xffff0000 {
                            return None;
                        }

                        self.output_buffer.push(0b00100000 | dest);
                        self.output_buffer.push(value as u8);
                        self.output_buffer.push((value >> 8) as u8);
                    }
                    "i" => {
                        let value: u32 = try_do!(self.parse_integral_value(items.next()));

                        self.output_buffer.push(0b00110000 | dest);
                        self.output_buffer.push((value >> 24) as u8);
                        self.output_buffer.push((value >> 16) as u8);
                        self.output_buffer.push((value >> 8) as u8);
                        self.output_buffer.push(value as u8);
                    }
                    "f" => {
                        let value: f32 = try_do!(result items.next()?.parse());
                        let value = unsafe { *(&value as *const f32 as *const u32) };
                        self.output_buffer.push(0b00110000 | dest);
                        self.output_buffer.push((value >> 24) as u8);
                        self.output_buffer.push((value >> 16) as u8);
                        self.output_buffer.push((value >> 8) as u8);
                        self.output_buffer.push(value as u8);
                    }
                    label => {
                        if !label.chars().all(Self::is_label_character) { return None; }

                        self.output_buffer.push(0b00110000 | dest);
                        self.label_placeholder_locations.push((
                            self.output_buffer.len(),
                            label.to_owned()
                        ));

                        // Push placeholder bytes for now
                        self.output_buffer.extend_from_slice(&[0, 0, 0, 0]);
                    }
                }
            }
            "halt" => {
                self.output_buffer.push(0b01000000);
            }
            "memcpy" => {
                // memcpy dst <- src ; size
                let dst = try_do!(Self::parse_location(items.next()));
                try_do!(items.next());
                let src = try_do!(Self::parse_location(items.next()));
                try_do!(items.next());
                let size = try_do!(Self::parse_location(items.next()));

                self.output_buffer.push(0b01010000 | size);
                self.output_buffer.push((src << 4) | dst);
            }
            "new" => {
                let dst = try_do!(Self::parse_location(items.next()));
                try_do!(items.next());
                let size = try_do!(Self::parse_location(items.next()));

                self.output_buffer.push(0b01100000);
                self.output_buffer.push((size << 4) | dst);
            }
            "del" => {
                let src = try_do!(Self::parse_location(items.next()));
                self.output_buffer.push(0b01110000 | src);
            }
            "neg" => try_do!(self.parse_and_emit_unop(&mut items, 0b1000)),
            "not" => try_do!(self.parse_and_emit_unop(&mut items, 0b1001)),
            "input" => {
                try_do!(items.next());
                let location = try_do!(Self::parse_location(items.next()));
                self.output_buffer.push(0b10100000 | location);
            }
            "output" => {
                try_do!(items.next());
                let location = try_do!(Self::parse_location(items.next()));
                self.output_buffer.push(0b10110000 | location);
            }
            _ => return None
        }

        Some(())
    }

    fn parse_and_emit_unop<'a>(&mut self, items: &mut impl Iterator<Item = &'a str>, opcode: u8) -> Option<()> {
        let ty = try_do!(Self::parse_type(items.next()));
        let dst = try_do!(Self::parse_location(items.next()));
        try_do!(items.next());
        try_do!(items.next());
        let src = try_do!(Self::parse_location(items.next()));

        self.output_buffer.push((opcode << 4) | (ty << 2));
        self.output_buffer.push((src << 4) | dst);

        Some(())
    }

    fn parse_integral_value(&self, x: Option<&str>) -> Option<u32> {
        let x = try_do!(x);

        if let Ok(value) = x.parse::<u32>() {
            return Some(value);
        }
        if let Ok(value) = x.parse::<i32>() {
            return Some(value as u32);
        }
        if x.starts_with("0x") {
            if let Ok(value) = u32::from_str_radix(&x[2..], 16) {
                return Some(value);
            }
        }

        None
    }

    fn parse_and_emit_binop<'a>(
        &mut self,
        items: &mut impl Iterator<Item = &'a str>,
        byte: u8,
    ) -> Option<()> {
        let x = items.next();
        let b1: u8 = 0b11000000u8 | (try_do!(Self::parse_type(x)) << 4) | byte;

        let dst_loc = try_do!(Self::parse_location(items.next()));
        try_do!(items.next());
        let src_loc_1 = try_do!(Self::parse_location(items.next()));
        try_do!(items.next());
        let src_loc_2 = try_do!(Self::parse_location(items.next()));

        let b2 = (src_loc_1 << 4) | src_loc_2;
        let b3 = dst_loc << 4;

        self.output_buffer.push(b1);
        self.output_buffer.push(b2);
        self.output_buffer.push(b3);

        Some(())
    }

    fn parse_type(x: Option<&str>) -> Option<u8> {
        Some(match try_do!(x) {
            "b" => 0b00u8,
            "s" => 0b01u8,
            "i" => 0b10u8,
            "f" => 0b11u8,
            _ => return None
        })
    }

    fn set_label_value(&mut self, command: &str) {
        let label_name = command[..command.len() - 1].to_string();
        self.label_locations.insert(label_name, self.output_buffer.len());
    }

    fn parse_location(location: Option<&str>) -> Option<u8> {
        Some(match try_do!(location) {
            "sp" => 0b0000,
            "@sp" => 0b1000,
            "r1" => 0b0001,
            "@r1" => 0b1001,
            "r2" => 0b0010,
            "@r2" => 0b1010,
            "r3" => 0b0011,
            "@r3" => 0b1011,
            "r4" => 0b0100,
            "@r4" => 0b1100,
            "r5" => 0b0101,
            "@r5" => 0b1101,
            "r6" => 0b0110,
            "@r6" => 0b1110,
            "r7" => 0b0111,
            "@r7" => 0b1111,
            _ => return None
        })
    }

    fn parse_condition(condition: Option<&str>) -> Option<u8> {
        Some(match try_do!(condition) {
            ">" => 0b0010,
            "==" => 0b0100,
            ">=" => 0b0110,
            "<" => 0b1000,
            "!=" => 0b1010,
            "<=" => 0b1100,
            _ => return None
        })
    }

    fn insert_labels(&mut self) -> Option<()> {
        for (label_location, label_name) in &self.label_placeholder_locations {
            let label_value = self.label_locations.get(label_name).expect(
                format!("Could not find label {}", label_name.as_str()).as_str()
            );
            self.output_buffer[*label_location + 3] = *label_value as u8;
            self.output_buffer[*label_location + 2] = (*label_value >> 8) as u8;
            self.output_buffer[*label_location + 1] = (*label_value >> 16) as u8;
            self.output_buffer[*label_location + 0] = (*label_value >> 24) as u8;
        }

        Some(())
    }

    fn is_label_character(c: char) -> bool {
        c.is_numeric() || c.is_uppercase() || c == '_'
    }
}