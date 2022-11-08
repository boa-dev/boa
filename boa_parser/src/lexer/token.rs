//! This module implements all of the [Token]s used in the JavaScript programing language.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-tokens

use crate::lexer::template::TemplateString;
use boa_ast::{Keyword, Punctuator, Span};
use boa_interner::{Interner, Sym};
use num_bigint::BigInt;

/// This represents the smallest individual words, phrases, or characters that JavaScript can understand.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-tokens
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
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
    #[must_use]
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Gets the kind of the token.
    #[inline]
    #[must_use]
    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }

    /// Gets the token span in the original source code.
    #[inline]
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }

    /// Converts the token to a `String`.
    pub(crate) fn to_string(&self, interner: &Interner) -> String {
        self.kind.to_string(interner)
    }
}

/// Represents the type different types of numeric literals.
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub enum Numeric {
    /// A floating point number
    Rational(f64),

    /// An integer
    Integer(i32),

    // A BigInt
    BigInt(Box<BigInt>),
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

impl From<BigInt> for Numeric {
    #[inline]
    fn from(n: BigInt) -> Self {
        Self::BigInt(Box::new(n))
    }
}

/// Represents the type of Token and the data it has inside.
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub enum TokenKind {
    /// A boolean literal, which is either `true` or `false`.
    BooleanLiteral(bool),

    /// The end of the file.
    EOF,

    /// An identifier.
    Identifier(Sym),

    /// A private identifier.
    PrivateIdentifier(Sym),

    /// A keyword and a flag if the keyword contains unicode escaped chars.
    Keyword((Keyword, bool)),

    /// A `null` literal.
    NullLiteral,

    /// A numeric literal.
    NumericLiteral(Numeric),

    /// A piece of punctuation
    Punctuator(Punctuator),

    /// A string literal.
    StringLiteral(Sym),

    /// A part of a template literal without substitution.
    TemplateNoSubstitution(TemplateString),

    /// The part of a template literal between substitutions
    TemplateMiddle(TemplateString),

    /// A regular expression, consisting of body and flags.
    RegularExpressionLiteral(Sym, Sym),

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

impl From<(Keyword, bool)> for TokenKind {
    fn from(kw: (Keyword, bool)) -> Self {
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
    #[must_use]
    pub fn boolean_literal(lit: bool) -> Self {
        Self::BooleanLiteral(lit)
    }

    /// Creates an `EOF` token kind.
    #[must_use]
    pub fn eof() -> Self {
        Self::EOF
    }

    /// Creates an `Identifier` token type.
    #[must_use]
    pub fn identifier(ident: Sym) -> Self {
        Self::Identifier(ident)
    }

    /// Creates a `NumericLiteral` token kind.
    pub fn numeric_literal<L>(lit: L) -> Self
    where
        L: Into<Numeric>,
    {
        Self::NumericLiteral(lit.into())
    }

    /// Creates a `Punctuator` token type.
    #[must_use]
    pub fn punctuator(punc: Punctuator) -> Self {
        Self::Punctuator(punc)
    }

    /// Creates a `StringLiteral` token type.
    #[must_use]
    pub fn string_literal(lit: Sym) -> Self {
        Self::StringLiteral(lit)
    }

    #[must_use]
    pub fn template_middle(template_string: TemplateString) -> Self {
        Self::TemplateMiddle(template_string)
    }

    #[must_use]
    pub fn template_no_substitution(template_string: TemplateString) -> Self {
        Self::TemplateNoSubstitution(template_string)
    }

    /// Creates a `RegularExpressionLiteral` token kind.
    #[must_use]
    pub fn regular_expression_literal(body: Sym, flags: Sym) -> Self {
        Self::RegularExpressionLiteral(body, flags)
    }

    /// Creates a `LineTerminator` token kind.
    #[must_use]
    pub fn line_terminator() -> Self {
        Self::LineTerminator
    }

    /// Creates a 'Comment' token kind.
    #[must_use]
    pub fn comment() -> Self {
        Self::Comment
    }

    /// Implements the `ToString` functionality for the `TokenKind`.
    #[must_use]
    pub fn to_string(&self, interner: &Interner) -> String {
        match *self {
            Self::BooleanLiteral(val) => val.to_string(),
            Self::EOF => "end of file".to_owned(),
            Self::Identifier(ident) => interner.resolve_expect(ident).to_string(),
            Self::PrivateIdentifier(ident) => format!("#{}", interner.resolve_expect(ident)),
            Self::Keyword((word, _)) => word.to_string(),
            Self::NullLiteral => "null".to_owned(),
            Self::NumericLiteral(Numeric::Rational(num)) => num.to_string(),
            Self::NumericLiteral(Numeric::Integer(num)) => num.to_string(),
            Self::NumericLiteral(Numeric::BigInt(ref num)) => format!("{num}n"),
            Self::Punctuator(punc) => punc.to_string(),
            Self::StringLiteral(lit) => interner.resolve_expect(lit).to_string(),
            Self::TemplateNoSubstitution(ts) | Self::TemplateMiddle(ts) => {
                interner.resolve_expect(ts.as_raw()).to_string()
            }
            Self::RegularExpressionLiteral(body, flags) => {
                format!(
                    "/{}/{}",
                    interner.resolve_expect(body),
                    interner.resolve_expect(flags),
                )
            }
            Self::LineTerminator => "line terminator".to_owned(),
            Self::Comment => "comment".to_owned(),
        }
    }
}
