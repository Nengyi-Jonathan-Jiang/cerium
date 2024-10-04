#[derive(Copy, Clone)]
pub enum CeCompareCondition {
    LT = 0b1000,
    EQ = 0b0100,
    LE = 0b1100,
    GT = 0b0010,
    NE = 0b1010,
    GE = 0b0110,
    Always = 0b1110,
}

#[derive(Copy, Clone)]
pub enum CeType {
    Int8 = 0,
    Int16 = 1,
    Int32 = 2,
    Float = 3,
}

#[derive(Copy, Clone)]
pub enum CeRegister {
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
pub struct CeLocation {
    register: CeRegister,
    indirect: bool,
}

impl CeLocation {
    pub fn to_u8(&self) -> u8 {
        let v = self.register as u8;
        if self.indirect { v & 0b1000 } else { v }
    }
}

pub enum CeBinaryOpCode {
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

pub enum CeUnaryOpCode {
    NEG = 0b1000,
    NOT = 0b1001,
}

pub enum CeInstruction {
    Mov {
        src_ty: CeType,
        dst_ty: CeType,
        src: CeLocation,
        dst: CeLocation,
    },
    Lod8(CeLocation, u8),
    Lod16(CeLocation, u16),
    Lod32(CeLocation, u32),
    Memcpy {
        src: CeLocation,
        dst: CeLocation,
        size: CeLocation,
    },
    New {
        size: CeLocation,
        dst: CeLocation,
    },
    Del {
        src: CeLocation,
    },
    BinOp {
        op: CeBinaryOpCode,
        ty: CeType,
        src1: CeLocation,
        src2: CeLocation,
        dst: CeLocation,
    },
    UnOp {
        op: CeUnaryOpCode,
        ty: CeType,
        src: CeLocation,
        dst: CeLocation,
    },
    Jmp {
        ty: CeType,
        src: CeLocation,
        tgt: CeLocation,
        cnd: CeCompareCondition,
    },
    Cmp {
        ty: CeType,
        src: CeLocation,
        dst: CeLocation,
        cnd: CeCompareCondition,
    },
    Input(CeLocation),
    Output(CeLocation),
}