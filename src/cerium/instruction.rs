pub mod casm_instruction_parts {
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

use casm_instruction_parts::*;

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
    Lod8(Location, u8),
    Lod16(Location, u16),
    Lod32(Location, u32),
    LodLabel(Location, String),
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