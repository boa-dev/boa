//! This module implements all of the [Token]s used in the JavaScript programing language.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-tokens

use super::regex::RegExpFlags;

use crate::{
    syntax::ast::{Keyword, Punctuator, Span},
    syntax::lexer::template::TemplateString,
    JsBigInt,
};
use std::fmt::{self, Debug, Display, Formatter};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// This represents the smallest individual words, phrases, or characters that JavaScript can understand.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-tokens
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The token kind, which contains the actual data of the token.
    kind: TokenKind,
    /// The token position in the original source code.
    span: Span,
}

impl Token {
    /// Create a new detailed token from the token data, line number and column number
    #[inline]
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Gets the kind of the token.
    #[inline]
    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    /// Gets the token span in the original source code.
    #[inline]
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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub enum Numeric {
    /// A floating point number
    Rational(f64),

    /// An integer
    Integer(i32),

    // A BigInt
    BigInt(JsBigInt),
}

impl From<f64> for Numeric {
    #[inline]
    fn from(n: f64) -> Self {
        Self::Rational(n)
    }
}

impl From<i32> for Numeric {
    #[inline]
    fn from(n: i32) -> Self {
        Self::Integer(n)
    }
}

impl From<JsBigInt> for Numeric {
    #[inline]
    fn from(n: JsBigInt) -> Self {
        Self::BigInt(n)
    }
}

/// Represents the type of Token and the data it has inside.
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
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
    NumericLiteral(Numeric),

    /// A piece of punctuation
    ///
    /// see: [`Punctuator`](../punc/enum.Punctuator.html)
    Punctuator(Punctuator),

    /// A string literal.
    StringLiteral(Box<str>),

    /// A part of a template literal without substitution.
    TemplateNoSubstitution(TemplateString),

    /// The part of a template literal between substitutions
    TemplateMiddle(TemplateString),

    /// A regular expression, consisting of body and flags.
    RegularExpressionLiteral(Box<str>, RegExpFlags),

    /// Indicates the end of a line (`\n`).
    LineTerminator,

    /// Indicates a comment, the content isn't stored.
    Comment,
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

impl From<Numeric> for TokenKind {
    fn from(num: Numeric) -> Self {
        Self::NumericLiteral(num)
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
        L: Into<Numeric>,
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

    pub fn template_middle(template_string: TemplateString) -> Self {
        Self::TemplateMiddle(template_string)
    }

    pub fn template_no_substitution(template_string: TemplateString) -> Self {
        Self::TemplateNoSubstitution(template_string)
    }

    /// Creates a `RegularExpressionLiteral` token kind.
    pub fn regular_expression_literal<B, R>(body: B, flags: R) -> Self
    where
        B: Into<Box<str>>,
        R: Into<RegExpFlags>,
    {
        Self::RegularExpressionLiteral(body.into(), flags.into())
    }

    /// Creates a `LineTerminator` token kind.
    pub fn line_terminator() -> Self {
        Self::LineTerminator
    }

    /// Creates a 'Comment' token kind.
    pub fn comment() -> Self {
        Self::Comment
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
            Self::NumericLiteral(Numeric::Rational(num)) => write!(f, "{}", num),
            Self::NumericLiteral(Numeric::Integer(num)) => write!(f, "{}", num),
            Self::NumericLiteral(Numeric::BigInt(ref num)) => write!(f, "{}n", num),
            Self::Punctuator(ref punc) => write!(f, "{}", punc),
            Self::StringLiteral(ref lit) => write!(f, "{}", lit),
            Self::TemplateNoSubstitution(ref ts) => write!(f, "{}", ts.as_raw()),
            Self::TemplateMiddle(ref ts) => write!(f, "{}", ts.as_raw()),
            Self::RegularExpressionLiteral(ref body, ref flags) => write!(f, "/{}/{}", body, flags),
            Self::LineTerminator => write!(f, "line terminator"),
            Self::Comment => write!(f, "comment"),
        }
    }
}
