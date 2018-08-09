use std::fmt::{Display, Formatter, Result};
use syntax::ast::keyword::Keyword;
use syntax::ast::pos::Position;

#[derive(Clone, PartialEq)]
pub struct Token {
    // // The token
    pub data: TokenData,
    pub pos: Position,
}

#[derive(Clone, PartialEq)]
pub enum TokenData {
    /// A boolean literal, which is either `true` or `false`
    TBooleanLiteral(bool),
    /// The end of the file
    TEOF,
    /// An identifier
    TIdentifier(String),
    /// A keyword
    TKeyword(Keyword),
    /// A `null` literal
    TNullLiteral,
    /// A numeric literal
    TNumericLiteral(f64),
    /// A piece of punctuation
    TPunctuator(Punctuator),
    /// A string literal
    TStringLiteral(String),
    /// A regular expression
    TRegularExpression(String),
    /// A comment
    TComment(String),
}

impl Display for TokenData {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self.clone() {
            TokenData::TBooleanLiteral(val) => write!(f, "{}", val),
            TEOF => write!(f, "end of file"),
            TokenData::TIdentifier(ident) => write!(f, "{}", ident),
            TokenData::TKeyword(word) => write!(f, "{}", word),
            TNullLiteral => write!(f, "null"),
            TokenData::TNumericLiteral(num) => write!(f, "{}", num),
            TokenData::TPunctuator(punc) => write!(f, "{}", punc),
            TokenData::TStringLiteral(lit) => write!(f, "{}", lit),
            TokenData::TRegularExpression(reg) => write!(f, "{}", reg),
            TokenData::TComment(comm) => write!(f, "/*{}*/", comm),
        }
    }
}
