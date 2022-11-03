//! The `Punctuator` enum, which contains all punctuators used in JavaScript.
//!
//! More information:
//!  - [ECMAScript Reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#prod-Punctuator

use crate::expression::operator::{
    assign::AssignOp,
    binary::{ArithmeticOp, BinaryOp, BitwiseOp, LogicalOp, RelationalOp},
};
use std::{
    convert::TryInto,
    fmt::{Display, Error, Formatter},
};

/// All of the punctuators used in JavaScript.
///
/// More information:
///  - [ECMAScript Reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-Punctuator
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Punctuator {
    /// `+`
    Add,
    /// `&`
    And,
    /// `=>`
    Arrow,
    /// `=`
    Assign,
    /// `+=`
    AssignAdd,
    /// `&=`
    AssignAnd,
    /// `&&=`
    AssignBoolAnd,
    /// `||=`
    AssignBoolOr,
    /// `??=`,
    AssignCoalesce,
    /// `/=`
    AssignDiv,
    /// `<<=`
    AssignLeftSh,
    /// `%=`
    AssignMod,
    /// `*=`
    AssignMul,
    /// `|=`
    AssignOr,
    /// `**=`
    AssignPow,
    /// `>>=`
    AssignRightSh,
    /// `-=`
    AssignSub,
    /// `>>>=`
    AssignURightSh,
    /// `^=`
    AssignXor,
    /// `&&`
    BoolAnd,
    /// `||`
    BoolOr,
    /// `}`
    CloseBlock,
    /// `]`
    CloseBracket,
    /// `)`
    CloseParen,
    /// `??`
    Coalesce,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `--`
    Dec,
    /// `/`
    Div,
    /// `.`
    Dot,
    /// `==`
    Eq,
    /// `>`
    GreaterThan,
    /// `>=`
    GreaterThanOrEq,
    /// `++`
    Inc,
    /// `<<`
    LeftSh,
    /// `<`
    LessThan,
    /// `<=`
    LessThanOrEq,
    /// `%`
    Mod,
    /// `*`
    Mul,
    /// `~`
    Neg,
    /// `!`
    Not,
    /// `!=`
    NotEq,
    /// `{`
    OpenBlock,
    /// `[`
    OpenBracket,
    /// `(`
    OpenParen,
    /// `?.`
    Optional,
    /// `|`
    Or,
    /// `**`
    Exp,
    /// `?`
    Question,
    /// `>>`
    RightSh,
    /// `;`
    Semicolon,
    /// `...`
    Spread,
    /// `===`
    StrictEq,
    /// `!==`
    StrictNotEq,
    /// `-`
    Sub,
    /// `>>>`
    URightSh,
    /// `^`
    Xor,
}

impl Punctuator {
    /// Attempts to convert a punctuator (`+`, `=`...) to an Assign Operator
    ///
    /// If there is no match, `None` will be returned.
    #[must_use]
    pub const fn as_assign_op(self) -> Option<AssignOp> {
        match self {
            Self::Assign => Some(AssignOp::Assign),
            Self::AssignAdd => Some(AssignOp::Add),
            Self::AssignAnd => Some(AssignOp::And),
            Self::AssignBoolAnd => Some(AssignOp::BoolAnd),
            Self::AssignBoolOr => Some(AssignOp::BoolOr),
            Self::AssignCoalesce => Some(AssignOp::Coalesce),
            Self::AssignDiv => Some(AssignOp::Div),
            Self::AssignLeftSh => Some(AssignOp::Shl),
            Self::AssignMod => Some(AssignOp::Mod),
            Self::AssignMul => Some(AssignOp::Mul),
            Self::AssignOr => Some(AssignOp::Or),
            Self::AssignPow => Some(AssignOp::Exp),
            Self::AssignRightSh => Some(AssignOp::Shr),
            Self::AssignSub => Some(AssignOp::Sub),
            Self::AssignURightSh => Some(AssignOp::Ushr),
            Self::AssignXor => Some(AssignOp::Xor),
            _ => None,
        }
    }

    /// Attempts to convert a punctuator (`+`, `=`...) to a Binary Operator
    ///
    /// If there is no match, `None` will be returned.
    #[must_use]
    pub const fn as_binary_op(self) -> Option<BinaryOp> {
        match self {
            Self::Add => Some(BinaryOp::Arithmetic(ArithmeticOp::Add)),
            Self::Sub => Some(BinaryOp::Arithmetic(ArithmeticOp::Sub)),
            Self::Mul => Some(BinaryOp::Arithmetic(ArithmeticOp::Mul)),
            Self::Div => Some(BinaryOp::Arithmetic(ArithmeticOp::Div)),
            Self::Mod => Some(BinaryOp::Arithmetic(ArithmeticOp::Mod)),
            Self::And => Some(BinaryOp::Bitwise(BitwiseOp::And)),
            Self::Or => Some(BinaryOp::Bitwise(BitwiseOp::Or)),
            Self::Xor => Some(BinaryOp::Bitwise(BitwiseOp::Xor)),
            Self::BoolAnd => Some(BinaryOp::Logical(LogicalOp::And)),
            Self::BoolOr => Some(BinaryOp::Logical(LogicalOp::Or)),
            Self::Coalesce => Some(BinaryOp::Logical(LogicalOp::Coalesce)),
            Self::Eq => Some(BinaryOp::Relational(RelationalOp::Equal)),
            Self::NotEq => Some(BinaryOp::Relational(RelationalOp::NotEqual)),
            Self::StrictEq => Some(BinaryOp::Relational(RelationalOp::StrictEqual)),
            Self::StrictNotEq => Some(BinaryOp::Relational(RelationalOp::StrictNotEqual)),
            Self::LessThan => Some(BinaryOp::Relational(RelationalOp::LessThan)),
            Self::GreaterThan => Some(BinaryOp::Relational(RelationalOp::GreaterThan)),
            Self::GreaterThanOrEq => Some(BinaryOp::Relational(RelationalOp::GreaterThanOrEqual)),
            Self::LessThanOrEq => Some(BinaryOp::Relational(RelationalOp::LessThanOrEqual)),
            Self::LeftSh => Some(BinaryOp::Bitwise(BitwiseOp::Shl)),
            Self::RightSh => Some(BinaryOp::Bitwise(BitwiseOp::Shr)),
            Self::URightSh => Some(BinaryOp::Bitwise(BitwiseOp::UShr)),
            Self::Comma => Some(BinaryOp::Comma),
            _ => None,
        }
    }

    /// Retrieves the punctuator as a static string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::And => "&",
            Self::Arrow => "=>",
            Self::Assign => "=",
            Self::AssignAdd => "+=",
            Self::AssignAnd => "&=",
            Self::AssignBoolAnd => "&&=",
            Self::AssignBoolOr => "||=",
            Self::AssignCoalesce => "??=",
            Self::AssignDiv => "/=",
            Self::AssignLeftSh => "<<=",
            Self::AssignMod => "%=",
            Self::AssignMul => "*=",
            Self::AssignOr => "|=",
            Self::AssignPow => "**=",
            Self::AssignRightSh => ">>=",
            Self::AssignSub => "-=",
            Self::AssignURightSh => ">>>=",
            Self::AssignXor => "^=",
            Self::BoolAnd => "&&",
            Self::BoolOr => "||",
            Self::Coalesce => "??",
            Self::CloseBlock => "}",
            Self::CloseBracket => "]",
            Self::CloseParen => ")",
            Self::Colon => ":",
            Self::Comma => ",",
            Self::Dec => "--",
            Self::Div => "/",
            Self::Dot => ".",
            Self::Eq => "==",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEq => ">=",
            Self::Inc => "++",
            Self::LeftSh => "<<",
            Self::LessThan => "<",
            Self::LessThanOrEq => "<=",
            Self::Mod => "%",
            Self::Mul => "*",
            Self::Neg => "~",
            Self::Not => "!",
            Self::NotEq => "!=",
            Self::OpenBlock => "{",
            Self::OpenBracket => "[",
            Self::OpenParen => "(",
            Self::Optional => "?.",
            Self::Or => "|",
            Self::Exp => "**",
            Self::Question => "?",
            Self::RightSh => ">>",
            Self::Semicolon => ";",
            Self::Spread => "...",
            Self::StrictEq => "===",
            Self::StrictNotEq => "!==",
            Self::Sub => "-",
            Self::URightSh => ">>>",
            Self::Xor => "^",
        }
    }
}

impl TryInto<BinaryOp> for Punctuator {
    type Error = String;
    fn try_into(self) -> Result<BinaryOp, Self::Error> {
        self.as_binary_op()
            .ok_or_else(|| format!("No binary operation for {self}"))
    }
}

impl Display for Punctuator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.as_str())
    }
}

impl From<Punctuator> for Box<str> {
    fn from(p: Punctuator) -> Self {
        p.as_str().into()
    }
}
