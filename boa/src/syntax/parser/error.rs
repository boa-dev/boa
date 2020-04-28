//! Error and result implementation for the parser.
use crate::syntax::ast::{
    keyword::Keyword,
    node::Node,
    pos::Position,
    token::{Token, TokenKind},
};
use std::fmt;

/// Result of a parsing operation.
pub type ParseResult = Result<Node, ParseError>;

/// `ParseError` is an enum which represents errors encounted during parsing an expression
#[derive(Debug, Clone)]
pub enum ParseError {
    /// When it expected a certain kind of token, but got another as part of something
    Expected(Vec<TokenKind>, Token, &'static str),
    /// When it expected a certain expression, but got another
    ExpectedExpr(&'static str, Node, Position),
    /// When it didn't expect this keyword
    UnexpectedKeyword(Keyword, Position),
    /// When a token is unexpected
    Unexpected(Token, Option<&'static str>),
    /// When there is an abrupt end to the parsing
    AbruptEnd,
    /// Out of range error, attempting to set a position where there is no token
    RangeError,
    /// Catch all General Error
    General(&'static str, Option<Position>),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expected(expected, actual, routine) => write!(
                f,
                "Expected {}, got '{}' in {} at line {}, col {}",
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
                actual,
                routine,
                actual.pos.line_number,
                actual.pos.column_number
            ),
            Self::ExpectedExpr(expected, actual, pos) => write!(
                f,
                "Expected expression '{}', got '{}' at line {}, col {}",
                expected, actual, pos.line_number, pos.column_number
            ),
            Self::UnexpectedKeyword(keyword, pos) => write!(
                f,
                "Unexpected keyword: '{}' at line {}, col {}",
                keyword, pos.line_number, pos.column_number
            ),
            Self::Unexpected(tok, msg) => write!(
                f,
                "Unexpected Token '{}'{} at line {}, col {}",
                tok,
                if let Some(m) = msg {
                    format!(", {}", m)
                } else {
                    String::new()
                },
                tok.pos.line_number,
                tok.pos.column_number
            ),
            Self::AbruptEnd => write!(f, "Abrupt End"),
            Self::General(msg, pos) => write!(
                f,
                "{}{}",
                msg,
                if let Some(pos) = pos {
                    format!(" at line {}, col {}", pos.line_number, pos.column_number)
                } else {
                    String::new()
                }
            ),
            Self::RangeError => write!(f, "RangeError!"),
        }
    }
}
