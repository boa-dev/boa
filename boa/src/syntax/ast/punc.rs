use std::fmt::{Display, Error, Formatter};

/// Punctuation
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
    Pow,
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
impl Display for Punctuator {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
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
                Punctuator::Pow => "**",
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
