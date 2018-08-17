use std::fmt::{Display, Error, Formatter};
#[derive(PartialEq, Clone, Debug)]
/// Punctuation
pub enum Punctuator {
    /// `{`
    OpenBlock,
    /// `}`
    CloseBlock,
    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// `[`
    OpenBracket,
    /// `]`
    CloseBracket,
    /// `.`
    Dot,
    /// `;`
    Semicolon,
    /// `,`
    Comma,
    /// `<`
    LessThan,
    /// `>`
    GreaterThan,
    /// `<=`
    LessThanOrEq,
    /// `>=`
    GreaterThanOrEq,
    /// `==`
    Eq,
    /// `!=`
    NotEq,
    /// `===`
    StrictEq,
    /// `!==`
    StrictNotEq,
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Mod,
    /// `++`
    Inc,
    /// `--`
    Dec,
    /// `<<`
    LeftSh,
    /// `>>`
    RightSh,
    /// `>>>`
    URightSh,
    /// `&`
    And,
    /// `|`
    Or,
    /// `^`
    Xor,
    /// `!`
    Not,
    /// `~`
    Neg,
    /// `&&`
    BoolAnd,
    /// `||`
    BoolOr,
    /// `?`
    Question,
    /// `:`
    Colon,
    /// `=`
    Assign,
    /// `+=`
    AssignAdd,
    /// `-=`
    AssignSub,
    /// `*=`
    AssignMul,
    /// `/=`
    AssignDiv,
    /// `%=`
    AssignMod,
    /// `<<=`
    AssignLeftSh,
    /// `>>=`
    AssignRightSh,
    /// `>>>=`
    AssignURightSh,
    /// `&=`
    AssignAnd,
    /// `|=`
    AssignOr,
    /// `^=`
    AssignXor,
    /// `=>`
    Arrow,
}
impl Display for Punctuator {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match self {
                Punctuator::OpenBlock => "{",
                Punctuator::CloseBlock => "}",
                Punctuator::OpenParen => "(",
                Punctuator::CloseParen => ")",
                Punctuator::OpenBracket => "[",
                Punctuator::CloseBracket => "]",
                Punctuator::Dot => ".",
                Punctuator::Semicolon => ";",
                Punctuator::Comma => ",",
                Punctuator::LessThan => "<",
                Punctuator::GreaterThan => ">",
                Punctuator::LessThanOrEq => "<=",
                Punctuator::GreaterThanOrEq => ">=",
                Punctuator::Eq => "==",
                Punctuator::NotEq => "!=",
                Punctuator::StrictEq => "===",
                Punctuator::StrictNotEq => "!==",
                Punctuator::Add => "+",
                Punctuator::Sub => "-",
                Punctuator::Mul => "*",
                Punctuator::Div => "/",
                Punctuator::Mod => "%",
                Punctuator::Inc => "++",
                Punctuator::Dec => "--",
                Punctuator::LeftSh => "<<",
                Punctuator::RightSh => ">>",
                Punctuator::URightSh => ">>>",
                Punctuator::And => "&",
                Punctuator::Or => "|",
                Punctuator::Xor => "^",
                Punctuator::Not => "!",
                Punctuator::Neg => "~",
                Punctuator::BoolAnd => "&&",
                Punctuator::BoolOr => "||",
                Punctuator::Question => "?",
                Punctuator::Colon => ":",
                Punctuator::Assign => "=",
                Punctuator::AssignAdd => "+=",
                Punctuator::AssignSub => "-=",
                Punctuator::AssignMul => "*=",
                Punctuator::AssignDiv => "/=",
                Punctuator::AssignMod => "%=",
                Punctuator::AssignLeftSh => "<<=",
                Punctuator::AssignRightSh => ">>=",
                Punctuator::AssignURightSh => ">>>=",
                Punctuator::AssignAnd => "&=",
                Punctuator::AssignOr => "|=",
                Punctuator::AssignXor => "^=",
                Punctuator::Arrow => "=>",
            }
        )
    }
}
