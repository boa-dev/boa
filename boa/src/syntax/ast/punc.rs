//! This module implements all punctuators used in ECMAScript
use crate::syntax::ast::op::{BinOp, BitOp, CompOp, LogOp, NumOp};
use std::fmt::{Display, Error, Formatter};

#[cfg(feature = "serde-ast")]
use serde::{Deserialize, Serialize};

/// The Punctuator enum describes all of the punctuators we use.
///
/// For more information [ECMAScript Reference](https://tc39.es/ecma262/#prod-Punctuator)
#[cfg_attr(feature = "serde-ast", derive(Serialize, Deserialize))]
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
            Punctuator::Add => Some(BinOp::Num(NumOp::Add)),
            Punctuator::Sub => Some(BinOp::Num(NumOp::Sub)),
            Punctuator::Mul => Some(BinOp::Num(NumOp::Mul)),
            Punctuator::Div => Some(BinOp::Num(NumOp::Div)),
            Punctuator::Mod => Some(BinOp::Num(NumOp::Mod)),
            Punctuator::And => Some(BinOp::Bit(BitOp::And)),
            Punctuator::Or => Some(BinOp::Bit(BitOp::Or)),
            Punctuator::Xor => Some(BinOp::Bit(BitOp::Xor)),
            Punctuator::BoolAnd => Some(BinOp::Log(LogOp::And)),
            Punctuator::BoolOr => Some(BinOp::Log(LogOp::Or)),
            Punctuator::Eq => Some(BinOp::Comp(CompOp::Equal)),
            Punctuator::NotEq => Some(BinOp::Comp(CompOp::NotEqual)),
            Punctuator::StrictEq => Some(BinOp::Comp(CompOp::StrictEqual)),
            Punctuator::StrictNotEq => Some(BinOp::Comp(CompOp::StrictNotEqual)),
            Punctuator::LessThan => Some(BinOp::Comp(CompOp::LessThan)),
            Punctuator::GreaterThan => Some(BinOp::Comp(CompOp::GreaterThan)),
            Punctuator::GreaterThanOrEq => Some(BinOp::Comp(CompOp::GreaterThanOrEqual)),
            Punctuator::LessThanOrEq => Some(BinOp::Comp(CompOp::LessThanOrEqual)),
            Punctuator::LeftSh => Some(BinOp::Bit(BitOp::Shl)),
            Punctuator::RightSh => Some(BinOp::Bit(BitOp::Shr)),
            Punctuator::URightSh => Some(BinOp::Bit(BitOp::UShr)),
            _ => None,
        }
    }
}

impl Display for Punctuator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match self {
                Punctuator::Add => "+",
                Punctuator::And => "&",
                Punctuator::Arrow => "=>",
                Punctuator::Assign => "=",
                Punctuator::AssignAdd => "+=",
                Punctuator::AssignAnd => "&=",
                Punctuator::AssignDiv => "/=",
                Punctuator::AssignLeftSh => "<<=",
                Punctuator::AssignMod => "%=",
                Punctuator::AssignMul => "*=",
                Punctuator::AssignOr => "|=",
                Punctuator::AssignPow => "**=",
                Punctuator::AssignRightSh => ">>=",
                Punctuator::AssignSub => "-=",
                Punctuator::AssignURightSh => ">>>=",
                Punctuator::AssignXor => "^=",
                Punctuator::BoolAnd => "&&",
                Punctuator::BoolOr => "||",
                Punctuator::CloseBlock => "}",
                Punctuator::CloseBracket => "]",
                Punctuator::CloseParen => ")",
                Punctuator::Colon => ":",
                Punctuator::Comma => ",",
                Punctuator::Dec => "--",
                Punctuator::Div => "/",
                Punctuator::Dot => ".",
                Punctuator::Eq => "==",
                Punctuator::GreaterThan => ">",
                Punctuator::GreaterThanOrEq => ">=",
                Punctuator::Inc => "++",
                Punctuator::LeftSh => "<<",
                Punctuator::LessThan => "<",
                Punctuator::LessThanOrEq => "<=",
                Punctuator::Mod => "%",
                Punctuator::Mul => "*",
                Punctuator::Neg => "~",
                Punctuator::Not => "!",
                Punctuator::NotEq => "!=",
                Punctuator::OpenBlock => "{",
                Punctuator::OpenBracket => "[",
                Punctuator::OpenParen => "(",
                Punctuator::Or => "|",
                Punctuator::Exp => "**",
                Punctuator::Question => "?",
                Punctuator::RightSh => ">>",
                Punctuator::Semicolon => ";",
                Punctuator::Spread => "...",
                Punctuator::StrictEq => "===",
                Punctuator::StrictNotEq => "!==",
                Punctuator::Sub => "-",
                Punctuator::URightSh => ">>>",
                Punctuator::Xor => "^",
            }
        )
    }
}
