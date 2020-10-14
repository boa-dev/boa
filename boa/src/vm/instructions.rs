#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Undefined,
    Null,
    Bool(bool),
    String(usize),
    BigInt(usize),

    /// Loads an i32 onto the stack
    Int32(i32),

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
}
