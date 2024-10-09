use crate::try_do;
use std::collections::HashMap;
use std::mem;
use crate::cerium::instruction::Instruction;
use crate::cerium::instruction::instruction_parts::{BinOp, UnOp, Condition, Location, Register, Type};

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

        use BinOp::*;
        use UnOp::*;

        match command {
            // Labels
            _ if command.chars().last().unwrap() == ':' && command.chars().rev().skip(1).all(
                Self::is_label_character
            ) => {
                self.set_label_value(command);
                return Some(());
            }

            // Arithmetic operations
            "xor" => self.parse_and_emit_binop(&mut items, XOR)?,
            "or" => self.parse_and_emit_binop(&mut items, OR)?,
            "and" => self.parse_and_emit_binop(&mut items, AND)?,
            "shl" => self.parse_and_emit_binop(&mut items, SHL)?,
            "shr" => self.parse_and_emit_binop(&mut items, SHR)?,
            "mul" => self.parse_and_emit_binop(&mut items, MUL)?,
            "add" => self.parse_and_emit_binop(&mut items, ADD)?,
            "sub" => self.parse_and_emit_binop(&mut items, SUB)?,
            "div" => self.parse_and_emit_binop(&mut items, DIV)?,
            "mod" => self.parse_and_emit_binop(&mut items, MOD)?,

            // Other operations
            "jmp" => {
                let tgt = Self::parse_location(items.next()?)?;
                let (ty, src, cnd) = match items.next()? {
                    "always" => (
                        Type::Int8,
                        Location { register: Register::SP, indirect: false },
                        Condition::ALWAYS
                    ),
                    "if" => (
                        Self::parse_ty(items.next()?)?,
                        Self::parse_location(items.next()?)?,
                        Self::parse_condition(items.next()?)?
                    ),
                    _ => return None
                };
                Instruction::Jmp { ty, src, tgt, cnd }
            }
            "cmp" => {
                let dst = Self::parse_location(items.next()?)?;

                items.next()?;

                let ty = Self::parse_ty(items.next()?)?;
                let src = Self::parse_location(items.next()?)?;
                let cnd = Self::parse_condition(items.next()?)?;

                Instruction::Cmp { ty, src, dst, cnd }
            }
            "mov" => {
                let dst_ty = Self::parse_ty(items.next()?)?;
                let dst = Self::parse_location(items.next()?)?;
                items.next()?;
                let src_ty = Self::parse_ty(items.next()?)?;
                let src = Self::parse_location(items.next()?)?;

                Instruction::Mov { src_ty, dst_ty, src, dst }
            }
            "lod" => {
                let dest = Self::parse_location(items.next()?)?;
                items.next()?;
                match items.next()? {
                    "b" => {
                        let value = self.parse_integral_value(items.next()?)?;
                        if (value & 0xffffff00) != 0 && (value & 0xffffff00) != 0xffffff00 {
                            return None;
                        }

                        Instruction::Lod8(dest, value as u8)
                    }
                    "s" => {
                        let value = self.parse_integral_value(items.next()?)?;
                        if (value & 0xffff0000) != 0 && (value & 0xffff0000) != 0xffff0000 {
                            return None;
                        }

                        Instruction::Lod16(dest, value as u16)
                    }
                    "i" => {
                        let value = self.parse_integral_value(items.next()?)?;

                        Instruction::Lod32(dest, value)
                    }
                    "f" => {
                        let value: f32 = try_do!(result items.next()?.parse());
                        let value = unsafe { mem::transmute::<f32, u32>(value) };

                        Instruction::Lod32(dest, value)
                    }
                    label => {
                        if !label.chars().all(Self::is_label_character) {
                            return None;
                        }

                        self.label_placeholder_locations.push((
                            self.output_buffer.len() + 1,
                            label.to_owned()
                        ));

                        Instruction::Lod32(dest, 0)
                    }
                }
            }
            "halt" => {
                Instruction::Halt
            }
            "memcpy" => {
                // memcpy dst <- src ; size
                let dst = Self::parse_location(items.next()?)?;
                items.next()?;
                let src = Self::parse_location(items.next()?)?;
                items.next()?;
                let size = Self::parse_location(items.next()?)?;

                Instruction::Memcpy { src, dst, size }
            }
            "new" => {
                let dst = Self::parse_location(items.next()?)?;
                items.next()?;
                let size = Self::parse_location(items.next()?)?;

                Instruction::New { size, dst }
            }
            "del" => {
                let src = Self::parse_location(items.next()?)?;
                Instruction::Del { src }
            }
            "neg" => self.parse_and_emit_unop(&mut items, NEG)?,
            "not" => self.parse_and_emit_unop(&mut items, NOT)?,
            "input" => {
                items.next()?;
                let location = Self::parse_location(items.next()?)?;
                Instruction::Input(location)
            }
            "output" => {
                items.next()?;
                let location = Self::parse_location(items.next()?)?;
                Instruction::Output(location)
            }
            _ => return None
        }.output_to(|x| self.write(x));

        Some(())
    }

    fn write(&mut self, x: u8) {
        self.output_buffer.push(x)
    }

    fn parse_and_emit_unop<'a>(
        &mut self,
        items: &mut impl Iterator<Item = &'a str>,
        op: UnOp,
    ) -> Option<Instruction> {
        let ty = Self::parse_ty(items.next()?)?;
        let dst = Self::parse_location(items.next()?)?;
        items.next()?;
        items.next()?;
        let src = Self::parse_location(items.next()?)?;

        Some(Instruction::UnOp {
            op,
            ty,
            src,
            dst,
        })
    }

    fn parse_integral_value(&self, x: &str) -> Option<u32> {
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
        op: BinOp,
    ) -> Option<Instruction> {
        let ty = Self::parse_ty(items.next()?)?;
        let dst = Self::parse_location(items.next()?)?;
        items.next()?;
        let src1 = Self::parse_location(items.next()?)?;
        items.next()?;
        let src2 = Self::parse_location(items.next()?)?;

        Some(Instruction::BinOp { op, ty, src1, src2, dst })
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

    fn set_label_value(&mut self, command: &str) {
        let label_name = command[..command.len() - 1].to_string();
        self.label_locations.insert(label_name, self.output_buffer.len());
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