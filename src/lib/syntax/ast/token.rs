use std::fmt::{Display, Formatter, Result};
use syntax::ast::keyword::Keyword;
use syntax::ast::pos::Position;
use syntax::ast::punc::Punctuator;

#[derive(Clone, PartialEq)]
/// Represents a token
pub struct Token {
    // // The token
    pub data: TokenData,
    pub pos: Position,
}

impl Token {
    /// Create a new detailed token from the token data, line number and column number
    pub fn new(data: TokenData, line_number: u64, column_number: u64) -> Token {
        Token {
            data: data,
            pos: Position::new(line_number, column_number),
        }
    }
}

#[derive(Clone, PartialEq)]
/// Represents the type of Token
pub enum TokenData {
    /// A boolean literal, which is either `true` or `false`
    BooleanLiteral(bool),
    /// The end of the file
    EOF,
    /// An identifier
    Identifier(String),
    /// A keyword
    Keyword(Keyword),
    /// A `null` literal
    NullLiteral,
    /// A numeric literal
    NumericLiteral(f64),
    /// A piece of punctuation
    Punctuator(Punctuator),
    /// A string literal
    StringLiteral(String),
    /// A regular expression
    RegularExpression(String),
    /// A comment
    Comment(String),
}

impl Display for TokenData {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self.clone() {
            TokenData::BooleanLiteral(val) => write!(f, "{}", val),
            TokenData::EOF => write!(f, "end of file"),
            TokenData::Identifier(ident) => write!(f, "{}", ident),
            TokenData::Keyword(word) => write!(f, "{}", word),
            TokenData::NullLiteral => write!(f, "null"),
            TokenData::NumericLiteral(num) => write!(f, "{}", num),
            TokenData::Punctuator(punc) => write!(f, "{}", punc),
            TokenData::StringLiteral(lit) => write!(f, "{}", lit),
            TokenData::RegularExpression(reg) => write!(f, "{}", reg),
            TokenData::Comment(comm) => write!(f, "/*{}*/", comm),
        }
    }
}
