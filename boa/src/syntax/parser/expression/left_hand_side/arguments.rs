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
        ast::{node::Spread, Node, Punctuator, Span},
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
    type Output = (Box<[Node]>, Span);

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("Arguments", "Parsing");

        let start_span = cursor
            .expect(Punctuator::OpenParen, "arguments")?
            .span()
            .start();
        let mut args = Vec::new();
        let end_span = loop {
            let next_token = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

            match next_token.kind() {
                TokenKind::Punctuator(Punctuator::CloseParen) => {
                    break cursor.next()?.expect(") token vanished").span().end();
                    // Consume the token.
                }
                TokenKind::Punctuator(Punctuator::Comma) => {
                    let next_token = cursor.next()?.expect(", token vanished"); // Consume the token.

                    if args.is_empty() {
                        return Err(ParseError::unexpected(next_token, None));
                    }

                    if let Some(tok) = cursor.next_if(Punctuator::CloseParen)? {
                        break tok.span().end();
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
                let assignment_expr =
                    AssignmentExpression::new(true, self.allow_yield, self.allow_await)
                        .parse(cursor)?;
                args.push(Node::new(
                    Spread::new(assignment_expr),
                    assignment_expr.span(),
                ));
            } else {
                cursor.set_goal(InputElement::RegExp);
                args.push(
                    AssignmentExpression::new(true, self.allow_yield, self.allow_await)
                        .parse(cursor)?,
                );
            }
        };

        let span = Span::new(start_span, end_span);
        Ok((args.into_boxed_slice(), span))
    }
}
