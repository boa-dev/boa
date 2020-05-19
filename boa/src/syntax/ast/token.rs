//! This module implements all of the [Token]s used in the JavaScript programing language.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-tokens

use crate::syntax::ast::{bigint::BigInt, keyword::Keyword, pos::Position, punc::Punctuator};
use std::fmt::{Debug, Display, Formatter, Result};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// This represents the smallest individual words, phrases, or characters that JavaScript can understand.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-tokens
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The token kind, which contains the actual data of the token.
    pub kind: TokenKind,

    /// The token position from origina source code.
    pub pos: Position,
}

impl Token {
    /// Create a new detailed token from the token data, line number and column number
    pub fn new(kind: TokenKind, line_number: u64, column_number: u64) -> Self {
        Self {
            kind,
            pos: Position::new(line_number, column_number),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.kind)
    }
}

/// A continuous sequence of tokens.
pub struct VecToken(Vec<Token>);

impl Debug for VecToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut buffer = String::new();
        for token in &self.0 {
            buffer.push_str(&token.to_string());
        }
        write!(f, "{}", buffer)
    }
}

/// Represents the type differenct types of numeric literals.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub enum NumericLiteral {
    /// A floating point number
    Rational(f64),

    /// An integer
    Integer(i32),

    // A BigInt
    BigInt(BigInt),
}

impl From<f64> for NumericLiteral {
    fn from(n: f64) -> Self {
        Self::Rational(n)
    }
}

impl From<i32> for NumericLiteral {
    fn from(n: i32) -> Self {
        Self::Integer(n)
    }
}

impl From<BigInt> for NumericLiteral {
    fn from(n: BigInt) -> Self {
        Self::BigInt(n)
    }
}

/// Represents the type of Token and the data it has inside.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub enum TokenKind {
    /// A boolean literal, which is either `true` or `false`.
    BooleanLiteral(bool),

    /// The end of the file.
    EOF,

    /// An identifier.
    Identifier(String),

    /// A keyword.
    ///
    /// see: [`Keyword`](../keyword/enum.Keyword.html)
    Keyword(Keyword),

    /// A `null` literal.
    NullLiteral,

    /// A numeric literal.
    NumericLiteral(NumericLiteral),

    /// A piece of punctuation
    ///
    /// see: [`Punctuator`](../punc/enum.Punctuator.html)
    Punctuator(Punctuator),

    /// A string literal.
    StringLiteral(String),

    /// A regular expression, consisting of body and flags.
    RegularExpressionLiteral(String, String),

    /// Indicates the end of a line (`\n`).
    LineTerminator,
}

impl From<bool> for TokenKind {
    fn from(oth: bool) -> Self {
        Self::BooleanLiteral(oth)
    }
}

impl From<Keyword> for TokenKind {
    fn from(kw: Keyword) -> Self {
        Self::Keyword(kw)
    }
}

impl From<Punctuator> for TokenKind {
    fn from(punc: Punctuator) -> Self {
        Self::Punctuator(punc)
    }
}

impl TokenKind {
    /// Creates a `BooleanLiteral` token kind.
    pub fn boolean_literal(lit: bool) -> Self {
        Self::BooleanLiteral(lit)
    }

    /// Creates an `EOF` token kind.
    pub fn eof() -> Self {
        Self::EOF
    }

    /// Creates an `Identifier` token type.
    pub fn identifier<I>(ident: I) -> Self
    where
        I: Into<String>,
    {
        Self::Identifier(ident.into())
    }

    /// Creates a `Keyword` token kind.
    pub fn keyword(keyword: Keyword) -> Self {
        Self::Keyword(keyword)
    }

    /// Creates a `NumericLiteral` token kind.
    pub fn numeric_literal<L>(lit: L) -> Self
    where
        L: Into<NumericLiteral>,
    {
        Self::NumericLiteral(lit.into())
    }

    /// Creates a `Punctuator` token type.
    pub fn punctuator(punc: Punctuator) -> Self {
        Self::Punctuator(punc)
    }

    /// Creates a `StringLiteral` token type.
    pub fn string_literal<S>(lit: S) -> Self
    where
        S: Into<String>,
    {
        Self::StringLiteral(lit.into())
    }

    /// Creates a `RegularExpressionLiteral` token kind.
    pub fn regular_expression_literal<B, F>(body: B, flags: F) -> Self
    where
        B: Into<String>,
        F: Into<String>,
    {
        Self::RegularExpressionLiteral(body.into(), flags.into())
    }

    /// Creates a `LineTerminator` token kind.
    pub fn line_terminator() -> Self {
        Self::LineTerminator
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match *self {
            Self::BooleanLiteral(ref val) => write!(f, "{}", val),
            Self::EOF => write!(f, "end of file"),
            Self::Identifier(ref ident) => write!(f, "{}", ident),
            Self::Keyword(ref word) => write!(f, "{}", word),
            Self::NullLiteral => write!(f, "null"),
            Self::NumericLiteral(NumericLiteral::Rational(num)) => write!(f, "{}", num),
            Self::NumericLiteral(NumericLiteral::Integer(num)) => write!(f, "{}", num),
            Self::NumericLiteral(NumericLiteral::BigInt(ref num)) => write!(f, "{}n", num),
            Self::Punctuator(ref punc) => write!(f, "{}", punc),
            Self::StringLiteral(ref lit) => write!(f, "{}", lit),
            Self::RegularExpressionLiteral(ref body, ref flags) => write!(f, "/{}/{}", body, flags),
            Self::LineTerminator => write!(f, "line terminator"),
        }
    }
}
