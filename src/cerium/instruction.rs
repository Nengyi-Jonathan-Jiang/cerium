pub mod instruction_parts {
    #[derive(Copy, Clone)]
    pub enum Condition {
        LT = 0b1000,
        EQ = 0b0100,
        LE = 0b1100,
        GT = 0b0010,
        NE = 0b1010,
        GE = 0b0110,
        ALWAYS = 0b1110,
    }

    #[derive(Copy, Clone)]
    pub enum Type {
        Int8 = 0,
        Int16 = 1,
        Int32 = 2,
        Float = 3,
    }

    impl Into<u8> for Type {
        fn into(self) -> u8 {
            self as u8
        }
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

    #[derive(Copy, Clone)]
    pub struct Location {
        pub(crate) register: Register,
        pub(crate) indirect: bool,
    }

    impl Location {
        pub(crate) fn as_u8(&self) -> u8 {
            let v = self.register as u8;
            if self.indirect { v | 0b1000 } else { v }
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

    #[derive(Copy, Clone)]
    pub enum UnOp {
        NEG = 0b1000,
        NOT = 0b1001,
    }
}

use instruction_parts::*;

#[derive(Copy, Clone)]
pub enum Instruction {
    Mov {
        src_ty: Type,
        dst_ty: Type,
        src: Location,
        dst: Location,
    },
    Lod8(Location, u8),
    Lod16(Location, u16),
    Lod32(Location, u32),
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
}

impl Instruction {
    pub fn output_to<F: FnMut(u8)>(&self, mut f: F) {
        use Instruction::*;

        match *self {
            Mov { src_ty, dst_ty, src, dst } => {
                f(((src_ty as u8) << 2) | (dst_ty as u8));
                f((src.as_u8() << 4) | dst.as_u8());
            }
            Lod8(loc, val) => {
                f(0b00010000 | loc.as_u8());
                f(val);
            }
            Lod16(loc, val) => {
                f(0b00100000 | loc.as_u8());
                f((val >> 8) as u8);
                f(val as u8);
            }
            Lod32(loc, val) => {
                f(0b00110000 | loc.as_u8());
                f((val >> 24) as u8);
                f((val >> 16) as u8);
                f((val >> 8) as u8);
                f(val as u8);
            }
            Halt => {
                f(0b01000000);
            }
            Memcpy { src, dst, size } => {
                f(0b01010000 | size.as_u8());
                f((src.as_u8() << 4) | dst.as_u8());
            }
            New { size, dst } => {
                f(0b01100000);
                f((size.as_u8() << 4) | dst.as_u8());
            }
            Del { src } => {
                f(0b01110000 | src.as_u8());
            }
            BinOp {
                op,
                ty,
                src1,
                src2,
                dst
            } => {
                f(0b11000000u8 | ((ty as u8) << 4) | (op as u8));
                f((src1.as_u8() << 4) | src2.as_u8());
                f(dst.as_u8() << 4);
            }
            UnOp { op, ty, src, dst } => {
                f(((op as u8) << 4) | ((ty as u8) << 2));
                f((src.as_u8() << 4) | dst.as_u8());
            }
            Cmp { ty, src, dst, cnd } => {
                f(0b11_00_1110_u8 | ((ty as u8) << 4));
                f((src.as_u8() << 4) | (cnd as u8));
                f(dst.as_u8() << 4);
            }
            Jmp { ty, src, tgt, cnd } => {
                f(0b11_00_1111_u8 | ((ty as u8) << 4));
                f((src.as_u8() << 4) | (cnd as u8));
                f(tgt.as_u8() << 4);
            }
            Input(dst) => f(0b10100000 | dst.as_u8()),
            Output(src) => f(0b10110000 | src.as_u8()),
        }
    }
}