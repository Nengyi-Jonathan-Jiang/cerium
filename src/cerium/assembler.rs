use crate::cerium::instruction::casm_instruction_parts::{
    BinOp, Condition, Location, Register, Type, UnOp,
};
use crate::cerium::instruction::CASMInstruction;
use crate::cerium::memory_buffer::EndianConversion;
use crate::cerium::vm::{CeInt16, CeInt32, CeInt8};
use std::collections::HashMap;
use std::{iter, mem};
use crate::cerium::cerium_error::CeriumAssemblerError;

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
            label_locations: Default::default(),
        };

        for instruction in instructions.iter().cloned() {
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

            match if line.starts_with("\"") {
                parse_str(&line[1..])
            } else {
                parse_line(line.split_whitespace())
            } {
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
        instruction.write_to_stream(|x: u8| self.output_buffer.push(x));

        match instruction {
            CASMInstruction::Label(label_name) => {
                self.label_locations
                    .insert(label_name, self.output_buffer.len());
            }
            CASMInstruction::ConstLabel(_location, label_name) => {
                self.label_placeholder_locations
                    .push((self.output_buffer.len() - 4, label_name));
            }
            _ => {}
        }
    }

    fn populate_label_placeholders(&mut self) -> Option<()> {
        for (label_location, label_name) in &self.label_placeholder_locations {
            let label_value = self
                .label_locations
                .get(label_name)
                .unwrap_or_else(|| {
                    CeriumAssemblerError::throw_string(format!("Could not find label {}", label_name.as_str()))
                });
            
            self.output_buffer[*label_location + 3] = *label_value as u8;
            self.output_buffer[*label_location + 2] = (*label_value >> 8) as u8;
            self.output_buffer[*label_location + 1] = (*label_value >> 16) as u8;
            self.output_buffer[*label_location + 0] = (*label_value >> 24) as u8;
        }

        Some(())
    }
}

fn parse_str(x: &str) -> Option<CASMInstruction> {
    let _ = x;
    todo!()
}

fn parse_line<'a>(mut items: impl Iterator<Item = &'a str>) -> Option<CASMInstruction> {
    let command = items.next()?;
    
    use self::BinOp::*;
    use self::UnOp::*;
    use CASMInstruction::*;

    Some(match command {
        // Labels
        _ if command.chars().last()? == ':'
            && command.chars().rev().skip(1).all(is_label_character) =>
        {
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
            // jmp [tgt] ( always | if [ty] [src] [cnd] )
            let tgt = parse_location(items.next()?)?;
            let (ty, src, cnd) = match items.next()? {
                "always" => (
                    Type::Int8,
                    // Any location will work, so just use 0 (sp)
                    Location::from_bits(0),
                    Condition::ALWAYS,
                ),
                "if" => (
                    parse_ty(items.next()?)?,
                    parse_location(items.next()?)?,
                    parse_condition(items.next()?)?,
                ),
                _ => return None,
            };
            Jmp { ty, src, tgt, cnd }
        }
        "cmp" => {
            // cmp [dst] <- [ty] [src] [cnd]

            let dst = parse_location(items.next()?)?;

            items.next()?;

            let ty = parse_ty(items.next()?)?;
            let src = parse_location(items.next()?)?;
            let cnd = parse_condition(items.next()?)?;

            Cmp { ty, src, dst, cnd }
        }
        "mov" => {
            // mov [dst_ty] [dst] <- ([src_ty] [src] | [constant])

            let dst_ty = parse_ty(items.next()?)?;
            let dst = parse_location(items.next()?)?;
            items.next()?;

            let src_item = items.next()?;
            if let Some(src_ty) = parse_ty(src_item) {
                // src is register
                let src = parse_location(items.next()?)?;

                Mov {
                    src_ty,
                    dst_ty,
                    src,
                    dst,
                }
            } else {
                // src is a constant
                match dst_ty {
                    Type::Int8 => Const8(dst, parse_i8(src_item)?),
                    Type::Int16 => Const16(dst, parse_i16(src_item)?),
                    Type::Int32 => {
                        if let Some(value) = parse_i32(src_item) {
                            // Integer constant
                            Const32(dst, value as CeInt32)
                        } else if src_item.chars().all(is_label_character) {
                            // For i32, we can also load labels
                            ConstLabel(dst, src_item.to_owned())
                        } else {
                            return None;
                        }
                    }
                    Type::Float => {
                        let value = parse_float_as_i32(src_item)?;
                        Const32(dst, value)
                    }
                }
            }
        }
        "halt" => Halt,
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
        _ => {
            // Raw data: ([type] [value] | [hex byte])*
            // Note that raw strings are handled separately outside of this function

            let mut items = iter::once(command).chain(items);
            let mut data = Vec::new();

            loop {
                let next = items.next();
                if let None = next {
                    break;
                }
                let next = unsafe {next.unwrap_unchecked() };
                // First try to parse data type + value
                if let Some(ty) = parse_ty(next) {
                    match ty {
                        Type::Int8 => data.extend(parse_i8(items.next()?)?.to_be_bytes()),
                        Type::Int16 => data.extend(parse_i16(items.next()?)?.to_be_bytes()),
                        Type::Int32 => data.extend(parse_i32(items.next()?)?.to_be_bytes()),
                        Type::Float => {
                            data.extend(parse_float_as_i32(items.next()?)?.to_be_bytes())
                        }
                    }
                }
                // Next try to parse as hex
                else if let Some(byte) = parse_hex_byte(next) {
                    data.push(byte);
                } else {
                    return None;
                }
            }
            Data(data.into_boxed_slice())
        }
    })
}

fn parse_hex_byte(x: &str) -> Option<u8> {
    if x.len() == 2 {
        if let Ok(byte) = u8::from_str_radix(x, 16) {
            Some(byte)
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_i32(x: &str) -> Option<u32> {
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

fn parse_i16(x: &str) -> Option<CeInt16> {
    let value = parse_i32(x)?;
    if (value & 0xffff0000) != 0 && (value & 0xffff0000) != 0xffff0000 {
        None
    } else {
        Some(value as CeInt16)
    }
}

fn parse_i8(x: &str) -> Option<CeInt8> {
    let value = parse_i32(x)?;

    if (value & 0xffffff00) != 0 && (value & 0xffffff00) != 0xffffff00 {
        None
    } else {
        Some(value as CeInt8)
    }
}

fn parse_float_as_i32(x: &str) -> Option<CeInt32> {
    if let Ok(value) = x.parse() {
        Some(unsafe { mem::transmute::<f32, CeInt32>(value) }.to_big_endian())
    } else {
        None
    }
}

fn parse_unop<'a>(items: &mut impl Iterator<Item = &'a str>, op: UnOp) -> Option<CASMInstruction> {
    let ty = parse_ty(items.next()?)?;
    let dst = parse_location(items.next()?)?;
    items.next()?;
    items.next()?;
    let src = parse_location(items.next()?)?;

    Some(CASMInstruction::UnOp { op, ty, src, dst })
}

fn parse_binop<'a>(
    items: &mut impl Iterator<Item = &'a str>,
    op: BinOp,
) -> Option<CASMInstruction> {
    let ty = parse_ty(items.next()?)?;
    let dst = parse_location(items.next()?)?;
    items.next()?;
    let src1 = parse_location(items.next()?)?;
    items.next()?;
    let src2 = parse_location(items.next()?)?;

    Some(CASMInstruction::BinOp {
        op,
        ty,
        src1,
        src2,
        dst,
    })
}

fn parse_ty(x: &str) -> Option<Type> {
    use Type::*;
    Some(match x {
        "b" => Int8,
        "s" => Int16,
        "i" => Int32,
        "f" => Float,
        _ => return None,
    })
}

fn parse_location(location: &str) -> Option<Location> {
    use Register::*;
    Some(match location {
        "sp" => Location::new(SP, false),
        "@sp" => Location::new(SP, true),
        "r1" => Location::new(R1, false),
        "@r1" => Location::new(R1, true),
        "r2" => Location::new(R2, false),
        "@r2" => Location::new(R2, true),
        "r3" => Location::new(R3, false),
        "@r3" => Location::new(R3, true),
        "r4" => Location::new(R4, false),
        "@r4" => Location::new(R4, true),
        "r5" => Location::new(R5, false),
        "@r5" => Location::new(R5, true),
        "r6" => Location::new(R6, false),
        "@r6" => Location::new(R6, true),
        "r7" => Location::new(R7, false),
        "@r7" => Location::new(R7, true),
        _ => return None,
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
        _ => return None,
    })
}

fn is_label_character(c: char) -> bool {
    c.is_numeric() || c.is_uppercase() || c == '_'
}
