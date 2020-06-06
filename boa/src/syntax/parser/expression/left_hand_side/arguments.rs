//! Argument parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/Argument
//! [spec]: https://tc39.es/ecma262/#prod-Arguments

use crate::{
    syntax::{
        ast::{Node, Punctuator, TokenKind},
        parser::{
            expression::AssignmentExpression, AllowAwait, AllowYield, Cursor, ParseError,
            TokenParser,
        },
    },
    BoaProfiler,
};

/// Parses a list of arguments.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/Argument
/// [spec]: https://tc39.es/ecma262/#prod-Arguments
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser::expression) struct Arguments {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl Arguments {
    /// Creates a new `Arguments` parser.
    pub(in crate::syntax::parser::expression) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl TokenParser for Arguments {
    type Output = Box<[Node]>;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("Arguments", "Parsing");
        cursor.expect(Punctuator::OpenParen, "arguments")?;
        let mut args = Vec::new();
        loop {
            let next_token = cursor.next().ok_or(ParseError::AbruptEnd)?;
            match next_token.kind {
                TokenKind::Punctuator(Punctuator::CloseParen) => break,
                TokenKind::Punctuator(Punctuator::Comma) => {
                    if args.is_empty() {
                        return Err(ParseError::unexpected(next_token.clone(), None));
                    }

                    if cursor.next_if(Punctuator::CloseParen).is_some() {
                        break;
                    }
                }
                _ => {
                    if !args.is_empty() {
                        return Err(ParseError::expected(
                            vec![
                                TokenKind::Punctuator(Punctuator::Comma),
                                TokenKind::Punctuator(Punctuator::CloseParen),
                            ],
                            next_token.clone(),
                            "argument list",
                        ));
                    } else {
                        cursor.back();
                    }
                }
            }

            if cursor.next_if(Punctuator::Spread).is_some() {
                args.push(Node::spread(
                    AssignmentExpression::new(true, self.allow_yield, self.allow_await)
                        .parse(cursor)?,
                ));
            } else {
                args.push(
                    AssignmentExpression::new(true, self.allow_yield, self.allow_await)
                        .parse(cursor)?,
                );
            }
        }
        Ok(args.into_boxed_slice())
    }
}
