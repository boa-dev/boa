//! Error and result implementation for the parser.

use crate::lexer::Error as LexError;
use boa_ast::{Position, Span};
use std::fmt;

/// Result of a parsing operation.
pub type ParseResult<T> = Result<T, Error>;

pub(crate) trait ErrorContext {
    fn context(self, context: &'static str) -> Self;
}

impl<T> ErrorContext for ParseResult<T> {
    #[inline]
    fn context(self, context: &'static str) -> Self {
        self.map_err(|e| e.context(context))
    }
}

impl From<LexError> for Error {
    #[inline]
    fn from(e: LexError) -> Self {
        Self::lex(e)
    }
}

/// An enum which represents errors encountered during parsing an expression
#[derive(Debug)]
pub enum Error {
    /// When it expected a certain kind of token, but got another as part of something
    Expected {
        /// The token(s) that were expected.
        expected: Box<[String]>,

        /// The token that was not expected.
        found: Box<str>,

        /// The parsing context in which the error occurred.
        context: &'static str,

        /// Position of the source code where the error occurred.
        span: Span,
    },

    /// When a token is unexpected
    Unexpected {
        /// The error message.
        message: Option<&'static str>,

        /// The token that was not expected.
        found: Box<str>,

        /// Position of the source code where the error occurred.
        span: Span,
    },

    /// When there is an abrupt end to the parsing
    AbruptEnd,

    /// A lexing error.
    Lex {
        /// The error that occurred during lexing.
        err: LexError,
    },

    /// Catch all General Error
    General {
        /// The error message.
        message: &'static str,

        /// Position of the source code where the error occurred.
        position: Position,
    },
}

impl Error {
    /// Changes the context of the error, if any.
    fn context(self, new_context: &'static str) -> Self {
        match self {
            Self::Expected {
                expected,
                found,
                span,
                ..
            } => Self::expected(expected, found, span, new_context),
            e => e,
        }
    }

    /// Creates an `Expected` parsing error.
    #[inline]
    pub(crate) fn expected<E, F>(expected: E, found: F, span: Span, context: &'static str) -> Self
    where
        E: Into<Box<[String]>>,
        F: Into<Box<str>>,
    {
        Self::Expected {
            expected: expected.into(),
            found: found.into(),
            span,
            context,
        }
    }

    /// Creates an `Expected` parsing error.
    #[inline]
    pub(crate) fn unexpected<F, C>(found: F, span: Span, message: C) -> Self
    where
        F: Into<Box<str>>,
        C: Into<Option<&'static str>>,
    {
        Self::Unexpected {
            found: found.into(),
            span,
            message: message.into(),
        }
    }

    /// Creates a "general" parsing error.
    #[inline]
    pub(crate) const fn general(message: &'static str, position: Position) -> Self {
        Self::General { message, position }
    }

    /// Creates a "general" parsing error with the specific error message for a wrong function declaration in non-strict mode.
    #[inline]
    pub(crate) const fn wrong_function_declaration_non_strict(position: Position) -> Self {
        Self::General {
            message: "In non-strict mode code, functions can only be declared at top level, inside a block, or as the body of an if statement.",
            position
        }
    }

    /// Creates a "general" parsing error with the specific error message for a wrong function declaration with label.
    #[inline]
    pub(crate) const fn wrong_labelled_function_declaration(position: Position) -> Self {
        Self::General {
            message: "Labelled functions can only be declared at top level or inside a block",
            position,
        }
    }

    /// Creates a parsing error from a lexing error.
    #[inline]
    pub(crate) const fn lex(e: LexError) -> Self {
        Self::Lex { err: e }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expected {
                expected,
                found,
                span,
                context,
            } => write!(
                f,
                "expected {}, got '{found}' in {context} at line {}, col {}",
                if expected.len() == 1 {
                    format!(
                        "token '{}'",
                        expected.first().expect("already checked that length is 1")
                    )
                } else {
                    format!(
                        "one of {}",
                        expected
                            .iter()
                            .enumerate()
                            .map(|(i, t)| {
                                format!(
                                    "{}'{t}'",
                                    if i == 0 {
                                        ""
                                    } else if i == expected.len() - 1 {
                                        " or "
                                    } else {
                                        ", "
                                    },
                                )
                            })
                            .collect::<String>()
                    )
                },
                span.start().line_number(),
                span.start().column_number()
            ),
            Self::Unexpected {
                found,
                span,
                message,
            } => write!(
                f,
                "unexpected token '{found}'{} at line {}, col {}",
                message.map_or_else(String::new, |m| format!(", {m}")),
                span.start().line_number(),
                span.start().column_number()
            ),
            Self::AbruptEnd => f.write_str("abrupt end"),
            Self::General { message, position } => write!(
                f,
                "{message} at line {}, col {}",
                position.line_number(),
                position.column_number()
            ),
            Self::Lex { err } => fmt::Display::fmt(err, f),
        }
    }
}
