#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Undefined,
    Null,
    True,
    False,
    Zero,
    One,
    String(usize),
    BigInt(usize),

    /// Loads an i32 onto the stack
    Int32(i32),

    /// Loads an f64 onto the stack
    Rational(f64),

    /// Adds the values from destination and source and stores the result in destination
    Add,

    /// subtracts the values from destination and source and stores the result in destination
    Sub,

    /// Multiplies the values from destination and source and stores the result in destination
    Mul,

    /// Divides the values from destination and source and stores the result in destination
    Div,

    Pow,

    Mod,

    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    UShr,

    Eq,
    NotEq,
    StrictEq,
    StrictNotEq,

    Gt,
    Ge,
    Lt,
    Le,

    In,
    InstanceOf,

    Void,
    TypeOf,
    Pos,
    Neg,
    BitNot,
    Not,

    /// The usize is the index of the variable name in the pool
    DefVar(usize),
    /// The usize is the index of the variable name in the pool
    DefLet(usize),
    /// The usize is the index of the variable name in the pool
    DefConst(usize),
    /// The usize is the index of the value to initiate the variable with in the pool
    InitLexical(usize),
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Undefined => write!(f, "Undefined"),
            Self::Null => write!(f, "Null"),
            Self::True => write!(f, "True"),
            Self::False => write!(f, "False"),
            Self::Zero => write!(f, "Zero"),
            Self::One => write!(f, "One"),
            Self::String(usize) => write!(f, "String({})", usize),
            Self::BigInt(usize) => write!(f, "BigInt({})", usize),
            Self::Int32(i32) => write!(f, "Int32({})", i32),
            Self::Rational(f64) => write!(f, "Rational({})", f64),
            Self::Add => write!(f, "Add"),
            Self::Sub => write!(f, "Sub"),
            Self::Mul => write!(f, "Mul"),
            Self::Div => write!(f, "Div"),
            Self::Pow => write!(f, "Pow"),
            Self::Mod => write!(f, "Mod"),
            Self::BitAnd => write!(f, "BitAnd"),
            Self::BitOr => write!(f, "BitOr"),
            Self::BitXor => write!(f, "BitXor"),
            Self::Shl => write!(f, "Shl"),
            Self::Shr => write!(f, "Shr"),
            Self::UShr => write!(f, "UShr"),
            Self::Eq => write!(f, "Eq"),
            Self::NotEq => write!(f, "NotEq"),
            Self::StrictEq => write!(f, "StrictEq"),
            Self::StrictNotEq => write!(f, "StrictNotEq"),
            Self::Gt => write!(f, "Gt"),
            Self::Ge => write!(f, "Ge"),
            Self::Lt => write!(f, "Lt"),
            Self::Le => write!(f, "Le"),
            Self::In => write!(f, "In"),
            Self::InstanceOf => write!(f, "InstanceOf"),
            Self::Void => write!(f, "Void"),
            Self::TypeOf => write!(f, "TypeOf"),
            Self::Pos => write!(f, "Pos"),
            Self::Neg => write!(f, "Neg"),
            Self::BitNot => write!(f, "BitNot"),
            Self::Not => write!(f, "Not"),
            Self::DefVar(name) => write!(f, "DefVar({})", name),
            Self::DefLet(name) => write!(f, "DefLet({})", name),
            Self::DefConst(name) => write!(f, "DefConst({})", name),
            Self::InitLexical(value) => write!(f, "InitLexical({})", value),
        }
    }
}
