use crate::syntax::ast::{keyword::Keyword, pos::Position, punc::Punctuator};
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Clone, PartialEq)]
/// Represents a token
#[derive(Debug)]
pub struct Token {
    /// The token Data
    pub data: TokenData,
    /// Token position from original source code
    pub pos: Position,
}

impl Token {
    /// Create a new detailed token from the token data, line number and column number
    pub fn new(data: TokenData, line_number: u64, column_number: u64) -> Self {
        Self {
            data,
            pos: Position::new(line_number, column_number),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.data)
    }
}

pub struct VecToken(Vec<Token>);

impl Debug for VecToken {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut buffer = String::new();
        for token in &self.0 {
            buffer.push_str(&token.to_string());
        }
        write!(f, "{}", buffer)
    }
}

#[derive(Clone, PartialEq, Debug)]
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
        match *self {
            TokenData::BooleanLiteral(ref val) => write!(f, "{}", val),
            TokenData::EOF => write!(f, "end of file"),
            TokenData::Identifier(ref ident) => write!(f, "{}", ident),
            TokenData::Keyword(ref word) => write!(f, "{}", word),
            TokenData::NullLiteral => write!(f, "null"),
            TokenData::NumericLiteral(ref num) => write!(f, "{}", num),
            TokenData::Punctuator(ref punc) => write!(f, "{}", punc),
            TokenData::StringLiteral(ref lit) => write!(f, "{}", lit),
            TokenData::RegularExpression(ref reg) => write!(f, "{}", reg),
            TokenData::Comment(ref comm) => write!(f, "/*{}*/", comm),
        }
    }
}
