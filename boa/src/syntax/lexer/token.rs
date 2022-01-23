//! This module implements all of the [Token]s used in the JavaScript programing language.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-tokens

use crate::{
    syntax::ast::{Keyword, Punctuator, Span},
    syntax::lexer::template::TemplateString,
    Interner, JsBigInt, Sym,
};

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

    /// Converts the token to a `String`.
    pub(crate) fn to_string(&self, interner: &Interner) -> String {
        self.kind.to_string(interner)
    }

    /// Converts the token to a string interner symbol.
    pub(crate) fn to_sym(&self, interner: &mut Interner) -> Sym {
        self.kind.to_sym(interner)
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
    Identifier(Sym),

    /// A keyword.
    Keyword(Keyword),

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
    pub fn identifier(ident: Sym) -> Self {
        Self::Identifier(ident)
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
    pub fn string_literal(lit: Sym) -> Self {
        Self::StringLiteral(lit)
    }

    pub fn template_middle(template_string: TemplateString) -> Self {
        Self::TemplateMiddle(template_string)
    }

    pub fn template_no_substitution(template_string: TemplateString) -> Self {
        Self::TemplateNoSubstitution(template_string)
    }

    /// Creates a `RegularExpressionLiteral` token kind.
    pub fn regular_expression_literal(body: Sym, flags: Sym) -> Self {
        Self::RegularExpressionLiteral(body, flags)
    }

    /// Creates a `LineTerminator` token kind.
    pub fn line_terminator() -> Self {
        Self::LineTerminator
    }

    /// Creates a 'Comment' token kind.
    pub fn comment() -> Self {
        Self::Comment
    }

    /// Implements the `ToString` functionality for the `TokenKind`.
    pub fn to_string(&self, interner: &Interner) -> String {
        match *self {
            Self::BooleanLiteral(val) => val.to_string(),
            Self::EOF => "end of file".to_owned(),
            Self::Identifier(ident) => interner.resolve_expect(ident).to_owned(),
            Self::Keyword(word) => word.to_string(),
            Self::NullLiteral => "null".to_owned(),
            Self::NumericLiteral(Numeric::Rational(num)) => num.to_string(),
            Self::NumericLiteral(Numeric::Integer(num)) => num.to_string(),
            Self::NumericLiteral(Numeric::BigInt(ref num)) => format!("{}n", num),
            Self::Punctuator(punc) => punc.to_string(),
            Self::StringLiteral(lit) => interner.resolve_expect(lit).to_owned(),
            Self::TemplateNoSubstitution(ts) | Self::TemplateMiddle(ts) => {
                interner.resolve_expect(ts.as_raw()).to_owned()
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

    /// Converts the token to a string interner symbol.
    ///
    /// This is an optimization to avoid resolving + re-interning strings.
    pub(crate) fn to_sym(&self, interner: &mut Interner) -> Sym {
        match *self {
            Self::BooleanLiteral(_)
            | Self::NumericLiteral(_)
            | Self::RegularExpressionLiteral(_, _) => {
                interner.get_or_intern(&self.to_string(interner))
            }
            Self::EOF => interner.get_or_intern_static("end of file"),
            Self::Identifier(sym) | Self::StringLiteral(sym) => sym,
            Self::Keyword(word) => interner.get_or_intern_static(word.as_str()),
            Self::NullLiteral => Sym::NULL,
            Self::Punctuator(punc) => interner.get_or_intern_static(punc.as_str()),
            Self::TemplateNoSubstitution(ts) | Self::TemplateMiddle(ts) => ts.as_raw(),
            Self::LineTerminator => interner.get_or_intern_static("line terminator"),
            Self::Comment => interner.get_or_intern_static("comment"),
        }
    }
}
