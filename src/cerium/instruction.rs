pub mod casm_instruction_parts {
    use crate::cerium::memory_buffer::CeriumPrimitiveType;
    use std::fmt::{Debug, Display};

    #[derive(Copy, Clone)]
    pub enum Condition {
        NEVER = 0b0000,
        LT = 0b1000,
        EQ = 0b0100,
        LE = 0b1100,
        GT = 0b0010,
        NE = 0b1010,
        GE = 0b0110,
        ALWAYS = 0b1110,
    }

    impl Condition {
        pub fn to_bits(&self) -> u8 {
            *self as u8
        }

        pub fn from_bits(bits: u8) -> Option<Condition> {
            Some(match bits & 0b1111 {
                0b0000 => Condition::NEVER,
                0b1000 => Condition::LT,
                0b0100 => Condition::EQ,
                0b1100 => Condition::LE,
                0b0010 => Condition::GT,
                0b1010 => Condition::NE,
                0b0110 => Condition::GE,
                0b1110 => Condition::ALWAYS,
                _ => return None,
            })
        }

        pub fn test<T: CeriumPrimitiveType>(&self, value: T) -> bool {
            let zero: T = 0.into();
            match self {
                Condition::NEVER => false,
                Condition::LT => value < zero,
                Condition::EQ => value == zero,
                Condition::LE => value <= zero,
                Condition::GT => value > zero,
                Condition::NE => value != zero,
                Condition::GE => value >= zero,
                Condition::ALWAYS => true,
            }
        }
    }

    impl Debug for Condition {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Display::fmt(
                match self {
                    Condition::NEVER => "never",
                    Condition::LT => "< 0",
                    Condition::EQ => "== 0",
                    Condition::LE => "<= 0",
                    Condition::GT => "> 0",
                    Condition::NE => "!= 0",
                    Condition::GE => ">= 0",
                    Condition::ALWAYS => "always",
                },
                f,
            )
        }
    }

    #[derive(Copy, Clone)]
    pub enum Type {
        Int8 = 0,
        Int16 = 1,
        Int32 = 2,
        Float = 3,
    }

    impl Default for Type {
        fn default() -> Self {
            Type::Int32
        }
    }

    impl Type {
        pub fn to_bits(&self) -> u8 {
            *self as u8
        }

        pub fn parse_from_bits(bits: u8) -> Option<Self> {
            Some(match bits & 0b11 {
                0b00 => Type::Int8,
                0b01 => Type::Int16,
                0b10 => Type::Int32,
                0b11 => Type::Float,
                _ => return None,
            })
        }
    }

    impl Debug for Type {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Display::fmt(
                match self {
                    Type::Int8 => "b",
                    Type::Int16 => "s",
                    Type::Int32 => "i",
                    Type::Float => "f",
                },
                f,
            )
        }
    }

    #[macro_export]
    macro_rules! match_cerium_type {
        (match $ty: expr => $func: ident ( $( $i:expr ), * )) => {
            {
                use casm_instruction_parts::Type::*;
                match $ty {
                    Int8  => $func::<CeInt8> ($( $i ), *),
                    Int16 => $func::<CeInt16>($( $i ), *),
                    Int32 => $func::<CeInt32>($( $i ), *),
                    Float => $func::<CeFloat>($( $i ), *),
                }
            }
        };
        (match $ty: ident => $func: ident ::< $( $t:ty ), * > ( $( $i:expr ), * )) => {
            {
                use casm_instruction_parts::Type::*;
                match $ty {
                    Int8  => $func::<$( $t ), *, CeInt8> ($( $i ), *),
                    Int16 => $func::<$( $t ), *, CeInt16>($( $i ), *),
                    Int32 => $func::<$( $t ), *, CeInt32>($( $i ), *),
                    Float => $func::<$( $t ), *, CeFloat>($( $i ), *),
                }
            }
        };
    }

    #[derive(Copy, Clone)]
    pub enum Register {
        SP = 0,
        R1 = 1,
        R2 = 2,
        R3 = 3,
        R4 = 4,
        R5 = 5,
        R6 = 6,
        R7 = 7,
    }

    impl Register {
        pub fn from_bits(bits: u8) -> Option<Self> {
            Some(match bits & 0b111 {
                0 => Register::SP,
                1 => Register::R1,
                2 => Register::R2,
                3 => Register::R3,
                4 => Register::R4,
                5 => Register::R5,
                6 => Register::R6,
                7 => Register::R7,
                _ => return None,
            })
        }

        pub fn to_bits(&self) -> u8 {
            *self as u8
        }
    }

    impl Debug for Register {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Register::SP => "sp",
                    Register::R1 => "r1",
                    Register::R2 => "r2",
                    Register::R3 => "r3",
                    Register::R4 => "r4",
                    Register::R5 => "r5",
                    Register::R6 => "r6",
                    Register::R7 => "r7",
                }
            )
        }
    }

    #[derive(Copy, Clone)]
    pub struct Location {
        pub register: Register,
        pub indirect: bool,
    }

    impl Location {
        pub fn to_bits(&self) -> u8 {
            let v = self.register.to_bits();
            if self.indirect {
                v | 0b1000
            } else {
                v
            }
        }

        pub fn from_bits(bits: u8) -> Option<Self> {
            Some(Self {
                register: Register::from_bits(bits)?,
                indirect: (bits & 0b1000) != 0,
            })
        }
    }

    impl Debug for Location {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}{:?}",
                if self.indirect { "@" } else { "" },
                self.register
            )
        }
    }

    #[derive(Copy, Clone)]
    pub enum BinOp {
        XOR = 0b0001,
        OR = 0b0010,
        AND = 0b0011,

        SHL = 0b0101,
        SHR = 0b0111,

        MUL = 0b1001,
        ADD = 0b1010,
        SUB = 0b1011,
        DIV = 0b1100,
        MOD = 0b1101,
    }

    impl BinOp {
        pub fn to_bits(&self) -> u8 {
            *self as u8
        }

        pub fn apply<T: CeriumPrimitiveType>(&self, a: T, b: T) -> T {
            match self {
                BinOp::XOR => <T as CeriumPrimitiveType>::xor(a, b),
                BinOp::OR => <T as CeriumPrimitiveType>::or(a, b),
                BinOp::AND => <T as CeriumPrimitiveType>::and(a, b),
                BinOp::SHL => <T as CeriumPrimitiveType>::shl(a, b),
                BinOp::SHR => <T as CeriumPrimitiveType>::shr(a, b),
                BinOp::MUL => <T as CeriumPrimitiveType>::mul(a, b),
                BinOp::ADD => <T as CeriumPrimitiveType>::add(a, b),
                BinOp::SUB => <T as CeriumPrimitiveType>::sub(a, b),
                BinOp::DIV => <T as CeriumPrimitiveType>::div(a, b),
                BinOp::MOD => <T as CeriumPrimitiveType>::rem(a, b),
            }
        }
    }

    #[derive(Copy, Clone)]
    pub enum UnOp {
        NEG = 0b1000,
        NOT = 0b1001,
    }

    impl UnOp {
        pub fn to_bits(&self) -> u8 {
            *self as u8
        }

        pub fn apply<T: CeriumPrimitiveType>(&self, a: T) -> T {
            match self {
                UnOp::NEG => <T as CeriumPrimitiveType>::neg(a),
                UnOp::NOT => <T as CeriumPrimitiveType>::not(a),
            }
        }
    }
}

use crate::cerium::memory_buffer::EndianConversion;
use crate::cerium::vm::{CeInt16, CeInt32, CeInt8};
use crate::util::ansi;
use casm_instruction_parts::*;
use std::fmt::{Debug, Formatter};
use std::hint::unreachable_unchecked;

#[derive(Clone)]
pub enum CASMInstruction {
    Data(Box<[u8]>),
    Label(String),
    Mov {
        src_ty: Type,
        dst_ty: Type,
        src: Location,
        dst: Location,
    },
    Const8(Location, CeInt8),
    Const16(Location, CeInt16),
    Const32(Location, CeInt32),
    ConstLabel(Location, String),
    Halt,
    Memcpy {
        src: Location,
        dst: Location,
        size: Location,
    },
    New {
        size: Location,
        dst: Location,
    },
    Del {
        src: Location,
    },
    Cmp {
        ty: Type,
        src: Location,
        dst: Location,
        cnd: Condition,
    },
    Jmp {
        ty: Type,
        src: Location,
        tgt: Location,
        cnd: Condition,
    },
    BinOp {
        op: BinOp,
        ty: Type,
        src1: Location,
        src2: Location,
        dst: Location,
    },
    UnOp {
        op: UnOp,
        ty: Type,
        src: Location,
        dst: Location,
    },
    Input(Location),
    Output(Location),
    NoOp,
}

impl Debug for CASMInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use ansi::colors::*;

        match self {
            CASMInstruction::Data(_) => todo!(),
            CASMInstruction::Label(_) => todo!(),

            CASMInstruction::Mov {
                src_ty,
                dst_ty,
                src,
                dst,
            } => {
                write!(
                    f,
                    "{}mov {}{:?} {}{:?} {}<- {}{:?} {}{:?}{}",
                    red(),
                    cyan(),
                    dst_ty,
                    green(),
                    dst,
                    red(),
                    cyan(),
                    src_ty,
                    green(),
                    src,
                    reset(),
                )
            }
            CASMInstruction::Const8(dst, dat) => {
                write!(
                    f,
                    "{}mov {}b {}{:?} {}<- {}{:?}{}",
                    red(),
                    cyan(),
                    green(),
                    dst,
                    red(),
                    purple(),
                    dat,
                    reset()
                )
            }
            CASMInstruction::Const16(dst, dat) => {
                write!(
                    f,
                    "{}mov {}s {}{:?} {}<- {}{:?}{}",
                    red(),
                    cyan(),
                    green(),
                    dst,
                    red(),
                    purple(),
                    dat,
                    reset()
                )
            }
            CASMInstruction::Const32(dst, dat) => {
                write!(
                    f,
                    "{}mov {}i {}{:?} {}<- {}{:?}{}",
                    red(),
                    cyan(),
                    green(),
                    dst,
                    red(),
                    purple(),
                    dat,
                    reset()
                )
            }
            CASMInstruction::ConstLabel(dst, dat) => {
                write!(
                    f,
                    "{}mov {}i {}{:?} {}<- {}{:?}",
                    red(),
                    cyan(),
                    green(),
                    dst,
                    red(),
                    reset(),
                    dat
                )
            }
            CASMInstruction::Halt => {
                write!(f, "{}halt{}", red(), reset())
            }
            CASMInstruction::Memcpy { src, dst, size } => {
                write!(
                    f,
                    "{}memcpy {}{:?} {}<- {}{:?} {}; {}{:?}{}",
                    red(),
                    green(),
                    dst,
                    red(),
                    green(),
                    src,
                    reset(),
                    green(),
                    size,
                    reset()
                )
            }
            CASMInstruction::New { size, dst } => {
                write!(
                    f,
                    "{}new {}{:?} {}; {}{:?}{}",
                    red(),
                    green(),
                    dst,
                    reset(),
                    green(),
                    size,
                    reset()
                )
            }
            CASMInstruction::Del { src } => {
                write!(f, "{}del {}{:?}{}", red(), green(), src, reset())
            }
            CASMInstruction::Cmp { ty, src, dst, cnd } => {
                write!(
                    f,
                    "{}cmp {}{:?} {}<- {}{:?} {}{:?} {}{:?}{}",
                    red(),
                    green(),
                    dst,
                    red(),
                    cyan(),
                    ty,
                    green(),
                    src,
                    red(),
                    cnd,
                    reset()
                )
            }
            CASMInstruction::Jmp { ty, src, tgt, cnd } => {
                if let Condition::ALWAYS = cnd {
                    write!(
                        f,
                        "{}jmp {}{:?} {}always{}",
                        red(),
                        green(),
                        tgt,
                        red(),
                        reset()
                    )
                } else {
                    write!(
                        f,
                        "{}jmp {}{:?} {}if {}{:?} {}{:?} {}{:?}{}",
                        red(),
                        green(),
                        tgt,
                        red(),
                        cyan(),
                        ty,
                        green(),
                        src,
                        red(),
                        cnd,
                        reset()
                    )
                }
            }
            CASMInstruction::BinOp {
                op,
                ty,
                src1,
                src2,
                dst,
            } => {
                write!(
                    f,
                    "{}{} {}{:?} {}{:?} {}<- {}{:?} {}{} {}{:?}{}",
                    red(),
                    match op {
                        BinOp::XOR => "xor",
                        BinOp::OR => "or",
                        BinOp::AND => "and",
                        BinOp::SHL => "shl",
                        BinOp::SHR => "shr",
                        BinOp::MUL => "mul",
                        BinOp::ADD => "add",
                        BinOp::SUB => "sub",
                        BinOp::DIV => "div",
                        BinOp::MOD => "mod",
                    },
                    cyan(),
                    ty,
                    green(),
                    dst,
                    red(),
                    green(),
                    src1,
                    red(),
                    match op {
                        BinOp::XOR => "^",
                        BinOp::OR => "|",
                        BinOp::AND => "&",
                        BinOp::SHL => "<<",
                        BinOp::SHR => ">>",
                        BinOp::MUL => "*",
                        BinOp::ADD => "+",
                        BinOp::SUB => "-",
                        BinOp::DIV => "/",
                        BinOp::MOD => "%",
                    },
                    green(),
                    src2,
                    reset()
                )
            }
            CASMInstruction::UnOp { op, ty, src, dst } => {
                write!(
                    f,
                    "{}{} {}{:?} {}{:?} {}<- {} {}{:?}{}",
                    red(),
                    match op {
                        UnOp::NEG => "neg",
                        UnOp::NOT => "not",
                    },
                    cyan(),
                    ty,
                    green(),
                    dst,
                    red(),
                    match op {
                        UnOp::NEG => "-",
                        UnOp::NOT => "~",
                    },
                    green(),
                    src,
                    reset()
                )
            }
            CASMInstruction::Input(dst) => {
                write!(f, "{}input -> {}{:?}{}", red(), green(), dst, reset())
            }
            CASMInstruction::Output(src) => {
                write!(f, "{}output <- {}{:?}{}", red(), green(), src, reset())
            }
            CASMInstruction::NoOp => {
                write!(f, "{}noop{}", red(), reset())
            }
        }
    }
}

impl CASMInstruction {
    pub(crate) fn parse_from_stream(
        stream: &mut impl CASMInstructionSourceStream,
    ) -> Option<CASMInstruction> {
        let curr_instruction_byte = stream.get_next::<u8>();

        Some(if (curr_instruction_byte >> 6) == 3 {
            // Ternary instructions
            let instruction_part = curr_instruction_byte & 0b00001111;

            let type_part = (curr_instruction_byte & 0b00110000) >> 4;
            let ty = Type::parse_from_bits(type_part)?;

            let b2 = stream.get_next::<u8>();
            let b3 = stream.get_next::<u8>();

            if let Some(op) = match instruction_part {
                0b0001 => Some(BinOp::XOR),
                0b0010 => Some(BinOp::OR),
                0b0011 => Some(BinOp::AND),
                0b0110 => Some(BinOp::SHL),
                0b0111 => Some(BinOp::SHR),
                0b1001 => Some(BinOp::MUL),
                0b1010 => Some(BinOp::ADD),
                0b1011 => Some(BinOp::SUB),
                0b1100 => Some(BinOp::DIV),
                0b1101 => Some(BinOp::MOD),
                _ => None,
            } {
                CASMInstruction::BinOp {
                    op,
                    ty,
                    src1: Location::from_bits(b2 >> 4)?,
                    src2: Location::from_bits(b2)?,
                    dst: Location::from_bits(b3 >> 4)?,
                }
            } else if instruction_part == 0b1110 {
                CASMInstruction::Cmp {
                    ty,
                    src: Location::from_bits(b2 >> 4)?,
                    dst: Location::from_bits(b3 >> 4)?,
                    cnd: Condition::from_bits(b2)?,
                }
            } else if instruction_part == 0b1111 {
                CASMInstruction::Jmp {
                    ty,
                    src: Location::from_bits(b2 >> 4)?,
                    tgt: Location::from_bits(b3 >> 4)?,
                    cnd: Condition::from_bits(b2)?,
                }
            } else {
                CASMInstruction::NoOp
            }
        } else {
            let instruction_part = curr_instruction_byte >> 4;
            match instruction_part {
                0b0000 => {
                    // MOV
                    let src_ty = Type::parse_from_bits(curr_instruction_byte >> 2)?;
                    let dst_ty = Type::parse_from_bits(curr_instruction_byte)?;

                    let b2 = stream.get_next::<u8>();
                    let src = Location::from_bits(b2 >> 4)?;
                    let dst = Location::from_bits(b2)?;

                    CASMInstruction::Mov {
                        src_ty,
                        dst_ty,
                        src,
                        dst,
                    }
                }
                0b0001 => {
                    // CONST8
                    let dat = stream.get_next::<CeInt8>();
                    CASMInstruction::Const8(
                        Location::from_bits(curr_instruction_byte)?,
                        dat,
                    )
                }
                0b0010 => {
                    // CONST16
                    let dat = stream.get_next::<CeInt16>();
                    CASMInstruction::Const16(
                        Location::from_bits(curr_instruction_byte)?,
                        dat,
                    )
                }
                0b0011 => {
                    // CONST32
                    let dat = stream.get_next::<CeInt32>();
                    CASMInstruction::Const32(
                        Location::from_bits(curr_instruction_byte)?,
                        dat,
                    )
                }
                0b0100 => CASMInstruction::Halt,
                0b0101 => {
                    // MEMCPY
                    let b2 = stream.get_next::<u8>();

                    CASMInstruction::Memcpy {
                        src: Location::from_bits(b2 >> 4)?,
                        dst: Location::from_bits(b2)?,
                        size: Location::from_bits(curr_instruction_byte)?,
                    }
                }
                0b0110 => {
                    // NEW
                    let b2 = stream.get_next::<u8>();

                    CASMInstruction::New {
                        size: Location::from_bits(b2 >> 4)?,
                        dst: Location::from_bits(b2)?,
                    }
                }
                0b0111 => {
                    // DEL
                    let b2 = stream.get_next::<u8>();

                    CASMInstruction::Del {
                        src: Location::from_bits(b2 >> 4)?,
                    }
                }
                0b1000 | 0b1001 => {
                    // NEG
                    let b2 = stream.get_next::<u8>();

                    CASMInstruction::UnOp {
                        op: match instruction_part {
                            0b1000 => UnOp::NEG,
                            0b1001 => UnOp::NOT,
                            _ => unsafe { unreachable_unchecked() },
                        },
                        ty: Type::parse_from_bits(curr_instruction_byte >> 2)?,
                        src: Location::from_bits(b2 >> 4)?,
                        dst: Location::from_bits(b2)?,
                    }
                }
                0b1010 => {
                    CASMInstruction::Input(Location::from_bits(curr_instruction_byte)?)
                }
                0b1011 => {
                    CASMInstruction::Output(Location::from_bits(curr_instruction_byte)?)
                }
                _ => unsafe { unreachable_unchecked() },
            }
        })
    }

    pub(crate) fn write_to_stream<F: FnMut(u8)>(&self, mut write_byte_to_output: F) {
        use CASMInstruction::*;

        match self {
            NoOp => write_byte_to_output(0b11_00_0000),
            Data(data) => {
                data.iter().cloned().for_each(|x| write_byte_to_output(x));
            }
            Label(_label_name) => {} // Don't do anything for labels
            Mov {
                src_ty,
                dst_ty,
                src,
                dst,
            } => {
                write_byte_to_output(((src_ty.to_bits()) << 2) | (dst_ty.to_bits()));
                write_byte_to_output((src.to_bits() << 4) | dst.to_bits());
            }
            Const8(loc, val) => {
                write_byte_to_output(0b00_01_0000 | loc.to_bits());
                write_byte_to_output(*val as u8);
            }
            Const16(loc, val) => {
                write_byte_to_output(0b00_10_0000 | loc.to_bits());
                write_byte_to_output((val >> 8) as u8);
                write_byte_to_output((val >> 0) as u8);
            }
            Const32(loc, val) => {
                write_byte_to_output(0b00_11_0000 | loc.to_bits());
                write_byte_to_output((val >> 24) as u8);
                write_byte_to_output((val >> 16) as u8);
                write_byte_to_output((val >> 8) as u8);
                write_byte_to_output((val >> 0) as u8);
            }
            ConstLabel(loc, _label_name) => {
                write_byte_to_output(0b00_11_0000 | loc.to_bits());
                write_byte_to_output(0);
                write_byte_to_output(0);
                write_byte_to_output(0);
                write_byte_to_output(0);
            }
            Halt => {
                write_byte_to_output(0b01000000);
            }
            Memcpy { src, dst, size } => {
                write_byte_to_output(0b01010000 | size.to_bits());
                write_byte_to_output((src.to_bits() << 4) | dst.to_bits());
            }
            New { size, dst } => {
                write_byte_to_output(0b01100000);
                write_byte_to_output((size.to_bits() << 4) | dst.to_bits());
            }
            Del { src } => {
                write_byte_to_output(0b01110000 | src.to_bits());
            }
            BinOp {
                op,
                ty,
                src1,
                src2,
                dst,
            } => {
                write_byte_to_output(0b11000000u8 | ((ty.to_bits()) << 4) | (op.to_bits()));
                write_byte_to_output((src1.to_bits() << 4) | src2.to_bits());
                write_byte_to_output(dst.to_bits() << 4);
            }
            UnOp { op, ty, src, dst } => {
                write_byte_to_output(((op.to_bits()) << 4) | ((ty.to_bits()) << 2));
                write_byte_to_output((src.to_bits() << 4) | dst.to_bits());
            }
            Cmp { ty, src, dst, cnd } => {
                write_byte_to_output(0b11_00_1110_u8 | ((ty.to_bits()) << 4));
                write_byte_to_output((src.to_bits() << 4) | (cnd.to_bits()));
                write_byte_to_output(dst.to_bits() << 4);
            }
            Jmp { ty, src, tgt, cnd } => {
                write_byte_to_output(0b11_00_1111_u8 | ((ty.to_bits()) << 4));
                write_byte_to_output((src.to_bits() << 4) | (cnd.to_bits()));
                write_byte_to_output(tgt.to_bits() << 4);
            }
            Input(dst) => write_byte_to_output(0b10100000 | dst.to_bits()),
            Output(src) => write_byte_to_output(0b10110000 | src.to_bits()),
        }
    }
}

pub trait CASMInstructionSourceStream {
    fn get_next<T: EndianConversion>(&mut self) -> T;
}
