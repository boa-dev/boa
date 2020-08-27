//! Error and result implementation for the parser.

use crate::syntax::ast::{position::Position, Node};
use crate::syntax::lexer::{Error as LexError, Token, TokenKind};
use std::fmt;

/// Result of a parsing operation.
pub type ParseResult = Result<Node, ParseError>;

pub(crate) trait ErrorContext {
    fn context(self, context: &'static str) -> Self;
}

impl<T> ErrorContext for Result<T, ParseError> {
    fn context(self, context: &'static str) -> Self {
        self.map_err(|e| e.context(context))
    }
}

impl From<LexError> for ParseError {
    fn from(e: LexError) -> ParseError {
        ParseError::lex(e)
    }
}

/// `ParseError` is an enum which represents errors encounted during parsing an expression
#[derive(Debug)]
pub enum ParseError {
    /// When it expected a certain kind of token, but got another as part of something
    Expected {
        expected: Box<[TokenKind]>,
        found: Token,
        context: &'static str,
    },
    /// When a token is unexpected
    Unexpected {
        found: Token,
        message: Option<&'static str>,
    },
    /// When there is an abrupt end to the parsing
    AbruptEnd,
    /// A lexing error.
    Lex { err: LexError },
    /// Catch all General Error
    General {
        message: &'static str,
        position: Position,
    },
}

impl ParseError {
    /// Changes the context of the error, if any.
    fn context(self, new_context: &'static str) -> Self {
        match self {
            Self::Expected {
                expected, found, ..
            } => Self::expected(expected, found, new_context),
            e => e,
        }
    }

    /// Creates an `Expected` parsing error.
    pub(super) fn expected<E>(expected: E, found: Token, context: &'static str) -> Self
    where
        E: Into<Box<[TokenKind]>>,
    {
        Self::Expected {
            expected: expected.into(),
            found,
            context,
        }
    }

    /// Creates an `Expected` parsing error.
    pub(super) fn unexpected<C>(found: Token, message: C) -> Self
    where
        C: Into<Option<&'static str>>,
    {
        Self::Unexpected {
            found,
            message: message.into(),
        }
    }

    /// Creates a "general" parsing error.
    pub(super) fn general(message: &'static str, position: Position) -> Self {
        Self::General { message, position }
    }

    /// Creates a parsing error from a lexing error.
    pub(super) fn lex(e: LexError) -> Self {
        Self::Lex { err: e }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expected {
                expected,
                found,
                context,
            } => write!(
                f,
                "expected {}, got '{}' in {} at line {}, col {}",
                if expected.len() == 1 {
                    format!(
                        "token '{}'",
                        expected.first().map(TokenKind::to_string).unwrap()
                    )
                } else {
                    format!(
                        "one of {}",
                        expected
                            .iter()
                            .enumerate()
                            .map(|(i, t)| {
                                format!(
                                    "{}'{}'",
                                    if i == 0 {
                                        ""
                                    } else if i == expected.len() - 1 {
                                        " or "
                                    } else {
                                        ", "
                                    },
                                    t
                                )
                            })
                            .collect::<String>()
                    )
                },
                found,
                context,
                found.span().start().line_number(),
                found.span().start().column_number()
            ),
            Self::Unexpected { found, message } => write!(
                f,
                "unexpected token '{}'{} at line {}, col {}",
                found,
                if let Some(m) = message {
                    format!(", {}", m)
                } else {
                    String::new()
                },
                found.span().start().line_number(),
                found.span().start().column_number()
            ),
            Self::AbruptEnd => f.write_str("abrupt end"),
            Self::General { message, position } => write!(
                f,
                "{} at line {}, col {}",
                message,
                position.line_number(),
                position.column_number()
            ),
            Self::Lex { err } => fmt::Display::fmt(err, f),
        }
    }
}
