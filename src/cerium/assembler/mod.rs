use crate::try_do;
use std::collections::HashMap;
use std::mem;
use crate::cerium::instruction::CASMInstruction;
use crate::cerium::instruction::casm_instruction_parts::{BinOp, UnOp, Condition, Location, Register, Type};

pub struct CeriumAssembler {
    output_buffer: Vec<u8>,
    label_placeholder_locations: Vec<(usize, String)>,
    label_locations: HashMap<String, usize>,
}

impl CeriumAssembler {
    pub fn assemble(instructions: Box<[CASMInstruction]>) -> Box<[u8]> {
        let mut assembler = CeriumAssembler {
            output_buffer: Default::default(),
            label_placeholder_locations: Default::default(),
            label_locations: Default::default()
        };
        
        for instruction in instructions {
            assembler.write_instruction(instruction)
        }
        
        assembler.populate_label_placeholders();
        assembler.output_buffer.into_boxed_slice()
    }

    pub fn parse_casm(source: &str) -> Box<[CASMInstruction]> {
        let mut instructions = Vec::<CASMInstruction>::new();

        for line in source.split("\n") {
            let line = line.trim();
            if line.starts_with("//") || line.is_empty() {
                continue;
            }

            match parse_line(line.split_whitespace()) {
                None => {
                    println!("Invalid line: {}", line)
                }
                Some(instruction) => {
                    instructions.push(instruction);
                }
            }
        }

        instructions.into_boxed_slice()
    }

    pub fn assemble_casm(source: &str) -> Box<[u8]> {
        Self::assemble(Self::parse_casm(source))
    }

    fn write_instruction(&mut self, instruction: CASMInstruction) {
        use CASMInstruction::*;

        match instruction {
            Data(data) => {
                data.iter().cloned().for_each(|x| self.write_byte_to_output(x));
            }
            Label(label_name) => {
                self.save_label_location(label_name);
            }
            Mov { src_ty, dst_ty, src, dst } => {
                self.write_byte_to_output(((src_ty as u8) << 2) | (dst_ty as u8));
                self.write_byte_to_output((src.as_u8() << 4) | dst.as_u8());
            }
            Lod8(loc, val) => {
                self.write_byte_to_output(0b00010000 | loc.as_u8());
                self.write_byte_to_output(val);
            }
            Lod16(loc, val) => {
                self.write_byte_to_output(0b00100000 | loc.as_u8());
                self.write_byte_to_output((val >> 8) as u8);
                self.write_byte_to_output(val as u8);
            }
            Lod32(loc, val) => {
                self.write_byte_to_output(0b00110000 | loc.as_u8());
                self.write_byte_to_output((val >> 24) as u8);
                self.write_byte_to_output((val >> 16) as u8);
                self.write_byte_to_output((val >> 8) as u8);
                self.write_byte_to_output(val as u8);
            }
            LodLabel(loc, label_name) => {
                self.write_byte_to_output(0b00110000 | loc.as_u8());

                self.add_label_placeholder(label_name);
            }
            Halt => {
                self.write_byte_to_output(0b01000000);
            }
            Memcpy { src, dst, size } => {
                self.write_byte_to_output(0b01010000 | size.as_u8());
                self.write_byte_to_output((src.as_u8() << 4) | dst.as_u8());
            }
            New { size, dst } => {
                self.write_byte_to_output(0b01100000);
                self.write_byte_to_output((size.as_u8() << 4) | dst.as_u8());
            }
            Del { src } => {
                self.write_byte_to_output(0b01110000 | src.as_u8());
            }
            BinOp {
                op,
                ty,
                src1,
                src2,
                dst
            } => {
                self.write_byte_to_output(0b11000000u8 | ((ty as u8) << 4) | (op as u8));
                self.write_byte_to_output((src1.as_u8() << 4) | src2.as_u8());
                self.write_byte_to_output(dst.as_u8() << 4);
            }
            UnOp { op, ty, src, dst } => {
                self.write_byte_to_output(((op as u8) << 4) | ((ty as u8) << 2));
                self.write_byte_to_output((src.as_u8() << 4) | dst.as_u8());
            }
            Cmp { ty, src, dst, cnd } => {
                self.write_byte_to_output(0b11_00_1110_u8 | ((ty as u8) << 4));
                self.write_byte_to_output((src.as_u8() << 4) | (cnd as u8));
                self.write_byte_to_output(dst.as_u8() << 4);
            }
            Jmp { ty, src, tgt, cnd } => {
                self.write_byte_to_output(0b11_00_1111_u8 | ((ty as u8) << 4));
                self.write_byte_to_output((src.as_u8() << 4) | (cnd as u8));
                self.write_byte_to_output(tgt.as_u8() << 4);
            }
            Input(dst) => self.write_byte_to_output(0b10100000 | dst.as_u8()),
            Output(src) => self.write_byte_to_output(0b10110000 | src.as_u8()),
        }
    }

    fn write_byte_to_output(&mut self, x: u8) {
        self.output_buffer.push(x)
    }

    fn save_label_location(&mut self, label_name: String) {
        self.label_locations.insert(label_name, self.output_buffer.len());
    }
    
    fn add_label_placeholder(&mut self, label_name: String) {
        self.label_placeholder_locations.push((
            self.output_buffer.len(),
            label_name
        ));
        self.write_byte_to_output(0);
        self.write_byte_to_output(0);
        self.write_byte_to_output(0);
        self.write_byte_to_output(0);
    }

    fn populate_label_placeholders(&mut self) -> Option<()> {
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
}

fn parse_line<'a>(mut items: impl Iterator<Item = &'a str>) -> Option<CASMInstruction> {
    let command = items.next().unwrap();

    use self::BinOp::*;
    use self::UnOp::*;
    use CASMInstruction::*;

    Some(match command {
        // Labels
        _ if command.chars().last().unwrap() == ':' && command.chars().rev().skip(1).all(
            is_label_character
        ) => {
            let label_name = &command[..command.len() - 1];
            Label(label_name.to_string())
        }

        // Arithmetic operations
        "xor" => parse_binop(&mut items, XOR)?,
        "or" => parse_binop(&mut items, OR)?,
        "and" => parse_binop(&mut items, AND)?,
        "shl" => parse_binop(&mut items, SHL)?,
        "shr" => parse_binop(&mut items, SHR)?,
        "mul" => parse_binop(&mut items, MUL)?,
        "add" => parse_binop(&mut items, ADD)?,
        "sub" => parse_binop(&mut items, SUB)?,
        "div" => parse_binop(&mut items, DIV)?,
        "mod" => parse_binop(&mut items, MOD)?,

        // Other operations
        "jmp" => {
            let tgt = parse_location(items.next()?)?;
            let (ty, src, cnd) = match items.next()? {
                "always" => (
                    Type::Int8,
                    Location { register: Register::SP, indirect: false },
                    Condition::ALWAYS
                ),
                "if" => (
                    parse_ty(items.next()?)?,
                    parse_location(items.next()?)?,
                    parse_condition(items.next()?)?
                ),
                _ => return None
            };
            Jmp { ty, src, tgt, cnd }
        }
        "cmp" => {
            let dst = parse_location(items.next()?)?;

            items.next()?;

            let ty = parse_ty(items.next()?)?;
            let src = parse_location(items.next()?)?;
            let cnd = parse_condition(items.next()?)?;

            Cmp { ty, src, dst, cnd }
        }
        "mov" => {
            let dst_ty = parse_ty(items.next()?)?;
            let dst = parse_location(items.next()?)?;
            items.next()?;
            let src_ty = parse_ty(items.next()?)?;
            let src = parse_location(items.next()?)?;

            Mov { src_ty, dst_ty, src, dst }
        }
        "lod" => {
            let dest = parse_location(items.next()?)?;
            items.next()?;
            match items.next()? {
                "b" => {
                    let value = parse_integral_value(items.next()?)?;
                    if (value & 0xffffff00) != 0 && (value & 0xffffff00) != 0xffffff00 {
                        return None;
                    }

                    Lod8(dest, value as u8)
                }
                "s" => {
                    let value = parse_integral_value(items.next()?)?;
                    if (value & 0xffff0000) != 0 && (value & 0xffff0000) != 0xffff0000 {
                        return None;
                    }

                    Lod16(dest, value as u16)
                }
                "i" => {
                    let value = parse_integral_value(items.next()?)?;

                    Lod32(dest, value)
                }
                "f" => {
                    let value: f32 = try_do!(result items.next()?.parse());
                    let value = unsafe { mem::transmute::<f32, u32>(value) };

                    Lod32(dest, value)
                }
                label => {
                    if !label.chars().all(is_label_character) {
                        return None;
                    }

                    LodLabel(dest, label.to_owned())
                }
            }
        }
        "halt" => {
            Halt
        }
        "memcpy" => {
            // memcpy dst <- src ; size
            let dst = parse_location(items.next()?)?;
            items.next()?;
            let src = parse_location(items.next()?)?;
            items.next()?;
            let size = parse_location(items.next()?)?;

            Memcpy { src, dst, size }
        }
        "new" => {
            // new l1 ; l2
            let dst = parse_location(items.next()?)?;
            items.next()?;
            let size = parse_location(items.next()?)?;

            New { size, dst }
        }
        "del" => {
            // del l1
            let src = parse_location(items.next()?)?;
            Del { src }
        }
        "neg" => parse_unop(&mut items, NEG)?,
        "not" => parse_unop(&mut items, NOT)?,
        "input" => {
            items.next()?;
            let location = parse_location(items.next()?)?;
            Input(location)
        }
        "output" => {
            items.next()?;
            let location = parse_location(items.next()?)?;
            Output(location)
        }
        _ => return None
    })
}

fn parse_unop<'a>(items: &mut impl Iterator<Item = &'a str>, op: UnOp) -> Option<CASMInstruction> {
    let ty = parse_ty(items.next()?)?;
    let dst = parse_location(items.next()?)?;
    items.next()?;
    items.next()?;
    let src = parse_location(items.next()?)?;

    Some(CASMInstruction::UnOp {
        op,
        ty,
        src,
        dst,
    })
}

fn parse_integral_value(x: &str) -> Option<u32> {
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

fn parse_binop<'a>(items: &mut impl Iterator<Item = &'a str>, op: BinOp) -> Option<CASMInstruction> {
    let ty = parse_ty(items.next()?)?;
    let dst = parse_location(items.next()?)?;
    items.next()?;
    let src1 = parse_location(items.next()?)?;
    items.next()?;
    let src2 = parse_location(items.next()?)?;

    Some(CASMInstruction::BinOp { op, ty, src1, src2, dst })
}

fn parse_ty(x: &str) -> Option<Type> {
    use Type::*;
    Some(match x {
        "b" => Int8,
        "s" => Int16,
        "i" => Int32,
        "f" => Float,
        _ => return None
    })
}

fn parse_location(location: &str) -> Option<Location> {
    use Register::*;
    Some(match location {
        "sp" => Location { register: SP, indirect: false },
        "@sp" => Location { register: SP, indirect: true },
        "r1" => Location { register: R1, indirect: false },
        "@r1" => Location { register: R1, indirect: true },
        "r2" => Location { register: R2, indirect: false },
        "@r2" => Location { register: R2, indirect: true },
        "r3" => Location { register: R3, indirect: false },
        "@r3" => Location { register: R3, indirect: true },
        "r4" => Location { register: R4, indirect: false },
        "@r4" => Location { register: R4, indirect: true },
        "r5" => Location { register: R5, indirect: false },
        "@r5" => Location { register: R5, indirect: true },
        "r6" => Location { register: R6, indirect: false },
        "@r6" => Location { register: R6, indirect: true },
        "r7" => Location { register: R7, indirect: false },
        "@r7" => Location { register: R7, indirect: true },
        _ => return None
    })
}

fn parse_condition(condition: &str) -> Option<Condition> {
    use Condition::*;
    Some(match condition {
        ">" => GT,
        "==" => EQ,
        ">=" => GE,
        "<" => LT,
        "!=" => NE,
        "<=" => LE,
        _ => return None
    })
}

fn is_label_character(c: char) -> bool {
    c.is_numeric() || c.is_uppercase() || c == '_'
}