//! Argument parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/Argument
//! [spec]: https://tc39.es/ecma262/#prod-Arguments

use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{node::Spread, Node, Punctuator},
        lexer::InputElement,
        parser::{
            expression::AssignmentExpression, AllowAwait, AllowYield, Cursor, ParseError,
            TokenParser,
        },
    },
    BoaProfiler,
};

use std::io::Read;

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

impl<R> TokenParser<R> for Arguments
where
    R: Read,
{
    type Output = Box<[Node]>;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("Arguments", "Parsing");

        cursor.expect(Punctuator::OpenParen, "arguments")?;
        let mut args = Vec::new();
        loop {
            let next_token = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

            match next_token.kind() {
                TokenKind::Punctuator(Punctuator::CloseParen) => {
                    cursor.next()?.expect(") token vanished"); // Consume the token.
                    break;
                }
                TokenKind::Punctuator(Punctuator::Comma) => {
                    let next_token = cursor.next()?.expect(", token vanished"); // Consume the token.

                    if args.is_empty() {
                        return Err(ParseError::unexpected(next_token, None));
                    }

                    if cursor.next_if(Punctuator::CloseParen)?.is_some() {
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
                    }
                }
            }

            if cursor.next_if(Punctuator::Spread)?.is_some() {
                args.push(
                    Spread::new(
                        AssignmentExpression::new(true, self.allow_yield, self.allow_await)
                            .parse(cursor)?,
                    )
                    .into(),
                );
            } else {
                cursor.set_goal(InputElement::RegExp);
                args.push(
                    AssignmentExpression::new(true, self.allow_yield, self.allow_await)
                        .parse(cursor)?,
                );
            }
        }
        Ok(args.into_boxed_slice())
    }
}
