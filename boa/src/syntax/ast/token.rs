//! This module implements all of the [Token]s used in the JavaScript programing language.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-tokens

use crate::builtins::BigInt;
use crate::syntax::{
    ast::{Keyword, Punctuator, Span},
    lexer::LexerError,
};
use bitflags::bitflags;
use std::{
    fmt::{self, Debug, Display, Formatter},
    str::FromStr,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
    pub(crate) kind: TokenKind,
    /// The token position in the original source code.
    span: Span,
}

impl Token {
    /// Create a new detailed token from the token data, line number and column number
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Gets the kind of the token.
    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    /// Gets the token span in the original source code.
    pub fn span(&self) -> Span {
        self.span
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
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

bitflags! {
    #[derive(Default)]
    pub struct RegExpFlags: u8 {
        const GLOBAL = 0b0000_0001;
        const IGNORE_CASE = 0b0000_0010;
        const MULTILINE = 0b0000_0100;
        const DOT_ALL = 0b0000_1000;
        const UNICODE = 0b0001_0000;
        const STICKY = 0b0010_0000;
    }
}

impl FromStr for RegExpFlags {
    type Err = LexerError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut flags = Self::default();
        for c in s.bytes() {
            let new_flag = match c {
                b'g' => Self::GLOBAL,
                b'i' => Self::IGNORE_CASE,
                b'm' => Self::MULTILINE,
                b's' => Self::DOT_ALL,
                b'u' => Self::UNICODE,
                b'y' => Self::STICKY,
                _ => {
                    return Err(LexerError::new(format!(
                        "invalid regular expression flag {}",
                        char::from(c)
                    )))
                }
            };

            if !flags.contains(new_flag) {
                flags.insert(new_flag);
            } else {
                return Err(LexerError::new(format!(
                    "invalid regular expression flag {}",
                    char::from(c)
                )));
            }
        }
        Ok(flags)
    }
}

impl Display for RegExpFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use fmt::Write;

        if self.contains(Self::GLOBAL) {
            f.write_char('g')?;
        }
        if self.contains(Self::IGNORE_CASE) {
            f.write_char('i')?;
        }
        if self.contains(Self::MULTILINE) {
            f.write_char('m')?;
        }
        if self.contains(Self::DOT_ALL) {
            f.write_char('s')?;
        }
        if self.contains(Self::UNICODE) {
            f.write_char('u')?;
        }
        if self.contains(Self::STICKY) {
            f.write_char('y')?;
        }
        Ok(())
    }
}

#[cfg(feature = "serde")]
impl Serialize for RegExpFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for RegExpFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        /// Deserializer visitor implementation for `RegExpFlags`.
        #[derive(Debug, Clone, Copy)]
        struct RegExpFlagsVisitor;

        impl<'de> Visitor<'de> for RegExpFlagsVisitor {
            type Value = RegExpFlags;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string representing JavaScript regular expression flags")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value.parse().map_err(E::custom)
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&value)
            }
        }

        deserializer.deserialize_str(RegExpFlagsVisitor)
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
    Identifier(Box<str>),

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
    StringLiteral(Box<str>),

    /// A regular expression, consisting of body and flags.
    RegularExpressionLiteral(Box<str>, RegExpFlags),

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
        I: Into<Box<str>>,
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
        S: Into<Box<str>>,
    {
        Self::StringLiteral(lit.into())
    }

    /// Creates a `RegularExpressionLiteral` token kind.
    pub fn regular_expression_literal<B>(body: B, flags: RegExpFlags) -> Self
    where
        B: Into<Box<str>>,
    {
        Self::RegularExpressionLiteral(body.into(), flags)
    }

    /// Creates a `LineTerminator` token kind.
    pub fn line_terminator() -> Self {
        Self::LineTerminator
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
