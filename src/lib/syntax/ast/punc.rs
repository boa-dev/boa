use std::fmt::{Display, Error, Formatter};
#[derive(PartialEq, Clone)]
/// Punctuation
pub enum Punctuator {
    /// `{`
    POpenBlock,
    /// `}`
    PCloseBlock,
    /// `(`
    POpenParen,
    /// `)`
    PCloseParen,
    /// `[`
    POpenBracket,
    /// `]`
    PCloseBracket,
    /// `.`
    PDot,
    /// `;`
    PSemicolon,
    /// `,`
    PComma,
    /// `<`
    PLessThan,
    /// `>`
    PGreaterThan,
    /// `<=`
    PLessThanOrEq,
    /// `>=`
    PGreaterThanOrEq,
    /// `==`
    PEq,
    /// `!=`
    PNotEq,
    /// `===`
    PStrictEq,
    /// `!==`
    PStrictNotEq,
    /// `+`
    PAdd,
    /// `-`
    PSub,
    /// `*`
    PMul,
    /// `/`
    PDiv,
    /// `%`
    PMod,
    /// `++`
    PInc,
    /// `--`
    PDec,
    /// `<<`
    PLeftSh,
    /// `>>`
    PRightSh,
    /// `>>>`
    PURightSh,
    /// `&`
    PAnd,
    /// `|`
    POr,
    /// `^`
    PXor,
    /// `!`
    PNot,
    /// `~`
    PNeg,
    /// `&&`
    PBoolAnd,
    /// `||`
    PBoolOr,
    /// `?`
    PQuestion,
    /// `:`
    PColon,
    /// `=`
    PAssign,
    /// `+=`
    PAssignAdd,
    /// `-=`
    PAssignSub,
    /// `*=`
    PAssignMul,
    /// `/=`
    PAssignDiv,
    /// `%=`
    PAssignMod,
    /// `<<=`
    PAssignLeftSh,
    /// `>>=`
    PAssignRightSh,
    /// `>>>=`
    PAssignURightSh,
    /// `&=`
    PAssignAnd,
    /// `|=`
    PAssignOr,
    /// `^=`
    PAssignXor,
    /// `=>`
    PArrow,
}
impl Display for Punctuator {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match self {
                Punctuator::POpenBlock => "{",
                Punctuator::PCloseBlock => "}",
                Punctuator::POpenParen => "(",
                Punctuator::PCloseParen => ")",
                Punctuator::POpenBracket => "[",
                Punctuator::PCloseBracket => "]",
                Punctuator::PDot => ".",
                Punctuator::PSemicolon => ";",
                Punctuator::PComma => ",",
                Punctuator::PLessThan => "<",
                Punctuator::PGreaterThan => ">",
                Punctuator::PLessThanOrEq => "<=",
                Punctuator::PGreaterThanOrEq => ">=",
                Punctuator::PEq => "==",
                Punctuator::PNotEq => "!=",
                Punctuator::PStrictEq => "===",
                Punctuator::PStrictNotEq => "!==",
                Punctuator::PAdd => "+",
                Punctuator::PSub => "-",
                Punctuator::PMul => "*",
                Punctuator::PDiv => "/",
                Punctuator::PMod => "%",
                Punctuator::PInc => "++",
                Punctuator::PDec => "--",
                Punctuator::PLeftSh => "<<",
                Punctuator::PRightSh => ">>",
                Punctuator::PURightSh => ">>>",
                Punctuator::PAnd => "&",
                Punctuator::POr => "|",
                Punctuator::PXor => "^",
                Punctuator::PNot => "!",
                Punctuator::PNeg => "~",
                Punctuator::PBoolAnd => "&&",
                Punctuator::PBoolOr => "||",
                Punctuator::PQuestion => "?",
                Punctuator::PColon => ":",
                Punctuator::PAssign => "=",
                Punctuator::PAssignAdd => "+=",
                Punctuator::PAssignSub => "-=",
                Punctuator::PAssignMul => "*=",
                Punctuator::PAssignDiv => "/=",
                Punctuator::PAssignMod => "%=",
                Punctuator::PAssignLeftSh => "<<=",
                Punctuator::PAssignRightSh => ">>=",
                Punctuator::PAssignURightSh => ">>>=",
                Punctuator::PAssignAnd => "&=",
                Punctuator::PAssignOr => "|=",
                Punctuator::PAssignXor => "^=",
                Punctuator::PArrow => "=>",
            }
        )
    }
}
