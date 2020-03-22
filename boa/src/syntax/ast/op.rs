use gc_derive::{Finalize, Trace};
use std::fmt::{Display, Formatter, Result};

#[cfg(feature = "serde-ast")]
use serde::{Deserialize, Serialize};

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

#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
/// A numeric operation between 2 values
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
    Exp,
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
                NumOp::Exp => "**",
                NumOp::Mod => "%",
            }
        )
    }
}

#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
/// A unary operation on a single value   
/// <https://tc39.es/ecma262/#prod-UnaryExpression>
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
    /// `type of` - Get the type of object
    TypeOf,
    /// `...a` - spread an iterable value
    Spread,
    /// Delete
    Delete,
    /// Void
    Void,
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
                UnaryOp::Delete => "delete",
                UnaryOp::TypeOf => "typeof",
                UnaryOp::Void => "void",
            }
        )
    }
}

#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
/// A bitwise operation between 2 values
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
    /// `a >>> b` - Zero-fill right shift
    UShr,
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
                BitOp::UShr => ">>>",
            }
        )
    }
}

#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
/// A comparitive operation between 2 values
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

#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
/// A logical operation between 2 boolean values
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

#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
/// A binary operation between 2 values
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
            BinOp::Num(NumOp::Exp) => 4,
            BinOp::Num(NumOp::Mul) | BinOp::Num(NumOp::Div) | BinOp::Num(NumOp::Mod) => 5,
            BinOp::Num(NumOp::Add) | BinOp::Num(NumOp::Sub) => 6,
            BinOp::Bit(BitOp::Shl) | BinOp::Bit(BitOp::Shr) | BinOp::Bit(BitOp::UShr) => 7,
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

#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
/// A binary operation between 2 values
/// https://tc39.es/ecma262/#prod-AssignmentOperator
pub enum AssignOp {
    /// `a += b` - Add assign
    Add,
    /// `a -= b` - Sub assign
    Sub,
    /// `a *= b` - Mul assign
    Mul,
    /// `a **= b` - Exponent assign
    Exp,
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
                AssignOp::Exp => "**=",
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
