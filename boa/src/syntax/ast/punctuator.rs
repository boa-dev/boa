//! This module implements the `Punctuator`, which represents all punctuators used in JavaScript
//!
//! More information:
//!  - [ECMAScript Reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#prod-Punctuator

use crate::syntax::ast::op::{AssignOp, BinOp, BitOp, CompOp, LogOp, NumOp};
use std::{
    convert::TryInto,
    fmt::{Display, Error, Formatter},
};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// The Punctuator enum describes all of the punctuators used in JavaScript.
///
/// More information:
///  - [ECMAScript Reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-Punctuator
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(PartialEq, Clone, Copy, Debug)]
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
    /// Attempts to convert a punctuator (`+`, `=`...) to a Binary Operator
    ///
    /// If there is no match, `None` will be returned.
    pub fn as_binop(self) -> Option<BinOp> {
        match self {
            Self::AssignAdd => Some(BinOp::Assign(AssignOp::Add)),
            Self::AssignAnd => Some(BinOp::Assign(AssignOp::And)),
            Self::AssignBoolAnd => Some(BinOp::Assign(AssignOp::BoolAnd)),
            Self::AssignBoolOr => Some(BinOp::Assign(AssignOp::BoolOr)),
            Self::AssignCoalesce => Some(BinOp::Assign(AssignOp::Coalesce)),
            Self::AssignDiv => Some(BinOp::Assign(AssignOp::Div)),
            Self::AssignLeftSh => Some(BinOp::Assign(AssignOp::Shl)),
            Self::AssignMod => Some(BinOp::Assign(AssignOp::Mod)),
            Self::AssignMul => Some(BinOp::Assign(AssignOp::Mul)),
            Self::AssignOr => Some(BinOp::Assign(AssignOp::Or)),
            Self::AssignPow => Some(BinOp::Assign(AssignOp::Exp)),
            Self::AssignRightSh => Some(BinOp::Assign(AssignOp::Shr)),
            Self::AssignSub => Some(BinOp::Assign(AssignOp::Sub)),
            Self::AssignURightSh => Some(BinOp::Assign(AssignOp::Ushr)),
            Self::AssignXor => Some(BinOp::Assign(AssignOp::Xor)),
            Self::Add => Some(BinOp::Num(NumOp::Add)),
            Self::Sub => Some(BinOp::Num(NumOp::Sub)),
            Self::Mul => Some(BinOp::Num(NumOp::Mul)),
            Self::Div => Some(BinOp::Num(NumOp::Div)),
            Self::Mod => Some(BinOp::Num(NumOp::Mod)),
            Self::And => Some(BinOp::Bit(BitOp::And)),
            Self::Or => Some(BinOp::Bit(BitOp::Or)),
            Self::Xor => Some(BinOp::Bit(BitOp::Xor)),
            Self::BoolAnd => Some(BinOp::Log(LogOp::And)),
            Self::BoolOr => Some(BinOp::Log(LogOp::Or)),
            Self::Coalesce => Some(BinOp::Log(LogOp::Coalesce)),
            Self::Eq => Some(BinOp::Comp(CompOp::Equal)),
            Self::NotEq => Some(BinOp::Comp(CompOp::NotEqual)),
            Self::StrictEq => Some(BinOp::Comp(CompOp::StrictEqual)),
            Self::StrictNotEq => Some(BinOp::Comp(CompOp::StrictNotEqual)),
            Self::LessThan => Some(BinOp::Comp(CompOp::LessThan)),
            Self::GreaterThan => Some(BinOp::Comp(CompOp::GreaterThan)),
            Self::GreaterThanOrEq => Some(BinOp::Comp(CompOp::GreaterThanOrEqual)),
            Self::LessThanOrEq => Some(BinOp::Comp(CompOp::LessThanOrEqual)),
            Self::LeftSh => Some(BinOp::Bit(BitOp::Shl)),
            Self::RightSh => Some(BinOp::Bit(BitOp::Shr)),
            Self::URightSh => Some(BinOp::Bit(BitOp::UShr)),
            Self::Comma => Some(BinOp::Comma),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
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

impl TryInto<BinOp> for Punctuator {
    type Error = String;
    fn try_into(self) -> Result<BinOp, Self::Error> {
        self.as_binop()
            .ok_or_else(|| format!("No binary operation for {}", self))
    }
}

impl Display for Punctuator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.as_str())
    }
}
