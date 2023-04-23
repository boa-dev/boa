//! Error and result implementation for the parser.

#[cfg(test)]
mod tests;

use crate::lexer::Error as LexError;
use boa_ast::{Position, Span};
use std::fmt;

/// Result of a parsing operation.
pub type ParseResult<T> = Result<T, Error>;

/// Adds context to a parser error.
pub(crate) trait ErrorContext {
    /// Sets the context of the error, if possible.
    fn set_context(self, context: &'static str) -> Self;

    /// Gets the context of the error, if any.
    fn context(&self) -> Option<&'static str>;
}

impl<T> ErrorContext for ParseResult<T> {
    fn set_context(self, context: &'static str) -> Self {
        self.map_err(|e| e.set_context(context))
    }

    fn context(&self) -> Option<&'static str> {
        self.as_ref().err().and_then(Error::context)
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
        message: Box<str>,

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
        message: Box<str>,

        /// Position of the source code where the error occurred.
        position: Position,
    },
}

impl Error {
    /// Changes the context of the error, if any.
    fn set_context(self, new_context: &'static str) -> Self {
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

    /// Gets the context of the error, if any.
    const fn context(&self) -> Option<&'static str> {
        if let Self::Expected { context, .. } = self {
            Some(context)
        } else {
            None
        }
    }

    /// Creates an `Expected` parsing error.
    pub(crate) fn expected<E, F>(expected: E, found: F, span: Span, context: &'static str) -> Self
    where
        E: Into<Box<[String]>>,
        F: Into<Box<str>>,
    {
        let expected = expected.into();
        debug_assert_ne!(expected.len(), 0);

        Self::Expected {
            expected,
            found: found.into(),
            span,
            context,
        }
    }

    /// Creates an `Unexpected` parsing error.
    pub(crate) fn unexpected<F, C>(found: F, span: Span, message: C) -> Self
    where
        F: Into<Box<str>>,
        C: Into<Box<str>>,
    {
        Self::Unexpected {
            found: found.into(),
            span,
            message: message.into(),
        }
    }

    /// Creates a "general" parsing error.
    pub(crate) fn general<S>(message: S, position: Position) -> Self
    where
        S: Into<Box<str>>,
    {
        Self::General {
            message: message.into(),
            position,
        }
    }

    /// Creates a "general" parsing error with the specific error message for a misplaced function declaration.
    pub(crate) fn misplaced_function_declaration(position: Position, strict: bool) -> Self {
        Self::General {
            message: format!(
                "{}functions can only be declared at the top level or inside a block.",
                if strict { "in strict mode code, " } else { "" }
            )
            .into(),
            position,
        }
    }

    /// Creates a "general" parsing error with the specific error message for a wrong function declaration with label.
    pub(crate) fn wrong_labelled_function_declaration(position: Position) -> Self {
        Self::General {
            message: "labelled functions can only be declared at the top level or inside a block"
                .into(),
            position,
        }
    }

    /// Creates a parsing error from a lexing error.
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
                "unexpected token '{found}', {message} at line {}, col {}",
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
            Self::Lex { err } => err.fmt(f),
        }
    }
}
