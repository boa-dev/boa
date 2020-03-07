use gc_derive::{Finalize, Trace};
use std::fmt::{Display, Formatter, Result};

/// Represents an operator
pub trait Operator {
    /// Get the associativity as a boolean that is true if it goes rightwards
    fn get_assoc(&self) -> bool;
    /// Get the precedence as an unsigned integer, where the lower it is, the more precedence/priority it has
    fn get_precedence(&self) -> u64;
    /// Get the precedence and associativity of this operator
    fn get_precedence_and_assoc(&self) -> (u64, bool) {
        (self.get_precedence(), self.get_assoc())
    }
}

/// A numeric operation between 2 values
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum NumOp {
    /// `a + b` - Addition
    Add,
    /// `a - b` - Subtraction
    Sub,
    /// `a / b` - Division
    Div,
    /// `a * b` - Multiplication
    Mul,
    /// `a ** b` - Exponentiation
    Pow,
    /// `a % b` - Modulus
    Mod,
}

impl Display for NumOp {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{}",
            match *self {
                NumOp::Add => "+",
                NumOp::Sub => "-",
                NumOp::Div => "/",
                NumOp::Mul => "*",
                NumOp::Pow => "**",
                NumOp::Mod => "%",
            }
        )
    }
}

/// A unary operation on a single value
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum UnaryOp {
    /// `a++` - increment the value
    IncrementPost,
    /// `++a` - increment the value
    IncrementPre,
    /// `a--` - decrement the value
    DecrementPost,
    /// `--a` - decrement the value
    DecrementPre,
    /// `-a` - negate the value
    Minus,
    /// `+a` - convert to a number
    Plus,
    /// `!a` - get the opposite of the boolean value
    Not,
    /// `~a` - bitwise-not of the value
    Tilde,
    /// `...a` - spread an iterable value
    Spread,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{}",
            match *self {
                UnaryOp::IncrementPost | UnaryOp::IncrementPre => "++",
                UnaryOp::DecrementPost | UnaryOp::DecrementPre => "--",
                UnaryOp::Plus => "+",
                UnaryOp::Minus => "-",
                UnaryOp::Not => "!",
                UnaryOp::Tilde => "~",
                UnaryOp::Spread => "...",
            }
        )
    }
}

/// A bitwise operation between 2 values
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum BitOp {
    /// `a & b` - Bitwise and
    And,
    /// `a | b` - Bitwise or
    Or,
    /// `a ^ b` - Bitwise xor
    Xor,
    /// `a << b` - Bit-shift leftwards
    Shl,
    /// `a >> b` - Bit-shift rightrights
    Shr,
}

impl Display for BitOp {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{}",
            match *self {
                BitOp::And => "&",
                BitOp::Or => "|",
                BitOp::Xor => "^",
                BitOp::Shl => "<<",
                BitOp::Shr => ">>",
            }
        )
    }
}

/// A comparitive operation between 2 values
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum CompOp {
    /// `a == b` - Equality
    Equal,
    /// `a != b` - Unequality
    NotEqual,
    /// `a === b` - Strict equality
    StrictEqual,
    /// `a !== b` - Strict unequality
    StrictNotEqual,
    /// `a > b` - If `a` is greater than `b`
    GreaterThan,
    /// `a >= b` - If `a` is greater than or equal to `b`
    GreaterThanOrEqual,
    /// `a < b` - If `a` is less than `b`
    LessThan,
    /// `a <= b` - If `a` is less than or equal to `b`
    LessThanOrEqual,
}

impl Display for CompOp {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{}",
            match *self {
                CompOp::Equal => "==",
                CompOp::NotEqual => "!=",
                CompOp::StrictEqual => "===",
                CompOp::StrictNotEqual => "!==",
                CompOp::GreaterThan => ">",
                CompOp::GreaterThanOrEqual => ">=",
                CompOp::LessThan => "<",
                CompOp::LessThanOrEqual => "<=",
            }
        )
    }
}

/// A logical operation between 2 boolean values
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum LogOp {
    /// `a && b` - Logical and
    And,
    /// `a || b` - Logical or
    Or,
}

impl Display for LogOp {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{}",
            match *self {
                LogOp::And => "&&",
                LogOp::Or => "||",
            }
        )
    }
}

/// A binary operation between 2 values
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum BinOp {
    /// Numeric operation
    Num(NumOp),
    /// Bitwise operation
    Bit(BitOp),
    /// Comparitive operation
    Comp(CompOp),
    /// Logical operation
    Log(LogOp),
    /// Assign operation
    Assign(AssignOp),
}

impl Operator for BinOp {
    fn get_assoc(&self) -> bool {
        true
    }
    fn get_precedence(&self) -> u64 {
        match *self {
            BinOp::Num(NumOp::Pow) => 4,
            BinOp::Num(NumOp::Mul) | BinOp::Num(NumOp::Div) | BinOp::Num(NumOp::Mod) => 5,
            BinOp::Num(NumOp::Add) | BinOp::Num(NumOp::Sub) => 6,
            BinOp::Bit(BitOp::Shl) | BinOp::Bit(BitOp::Shr) => 7,
            BinOp::Comp(CompOp::LessThan)
            | BinOp::Comp(CompOp::LessThanOrEqual)
            | BinOp::Comp(CompOp::GreaterThan)
            | BinOp::Comp(CompOp::GreaterThanOrEqual) => 8,
            BinOp::Comp(CompOp::Equal)
            | BinOp::Comp(CompOp::NotEqual)
            | BinOp::Comp(CompOp::StrictEqual)
            | BinOp::Comp(CompOp::StrictNotEqual) => 9,
            BinOp::Bit(BitOp::And) => 10,
            BinOp::Bit(BitOp::Xor) => 11,
            BinOp::Bit(BitOp::Or) => 12,
            BinOp::Log(LogOp::And) => 13,
            BinOp::Log(LogOp::Or) => 14,
            BinOp::Assign(_) => 15,
        }
    }
}

impl Display for BinOp {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{}",
            match *self {
                BinOp::Num(ref op) => op.to_string(),
                BinOp::Bit(ref op) => op.to_string(),
                BinOp::Comp(ref op) => op.to_string(),
                BinOp::Log(ref op) => op.to_string(),
                BinOp::Assign(ref op) => op.to_string(),
            }
        )
    }
}

/// A binary operation between 2 values
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum AssignOp {
    /// `a += b` - Add assign
    Add,
    /// `a -= b` - Sub assign
    Sub,
    /// `a *= b` - Mul assign
    Mul,
    /// `a **= b` - Exponent assign
    Pow,
    /// `a /= b` - Div assign
    Div,
    /// `a %= b` - Modulus assign
    Mod,
    /// `a &= b` - Bitwise and assign
    And,
    /// `a |= b` - Bitwise or assign
    Or,
    /// `a ^= b` - Bitwise xor assign
    Xor,
    /// `a <<= b` - Left shift assign
    Shl,
    /// `a >>= b` - Right shift assign
    Shr,
}

impl Display for AssignOp {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{}",
            match *self {
                AssignOp::Add => "+=",
                AssignOp::Sub => "-=",
                AssignOp::Mul => "*=",
                AssignOp::Pow => "**=",
                AssignOp::Div => "/=",
                AssignOp::Mod => "%=",
                AssignOp::And => "&=",
                AssignOp::Or => "|=",
                AssignOp::Xor => "^=",
                AssignOp::Shl => "<<=",
                AssignOp::Shr => ">>=",
            }
        )
    }
}
