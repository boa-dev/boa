//! Argument parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/Argument
//! [spec]: https://tc39.es/ecma262/#prod-Arguments

use crate::{
    lexer::{InputElement, TokenKind},
    parser::{
        expression::AssignmentExpression, AllowAwait, AllowYield, Cursor, OrAbrupt, ParseResult,
        TokenParser,
    },
    Error,
};
use boa_ast::{expression::Spread, Expression, Punctuator};
use boa_interner::Interner;
use boa_profiler::Profiler;
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
pub(in crate::parser::expression) struct Arguments {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl Arguments {
    /// Creates a new `Arguments` parser.
    pub(in crate::parser::expression) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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
    type Output = Box<[Expression]>;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("Arguments", "Parsing");

        cursor.expect(Punctuator::OpenParen, "arguments", interner)?;
        let mut args = Vec::new();
        loop {
            cursor.set_goal(InputElement::RegExp);
            let next_token = cursor.peek(0, interner).or_abrupt()?;

            match next_token.kind() {
                TokenKind::Punctuator(Punctuator::CloseParen) => {
                    cursor.advance(interner);
                    break;
                }
                TokenKind::Punctuator(Punctuator::Comma) => {
                    let next_token = cursor.next(interner)?.expect(", token vanished"); // Consume the token.

                    if args.is_empty() {
                        return Err(Error::unexpected(
                            next_token.to_string(interner),
                            next_token.span(),
                            None,
                        ));
                    }

                    if cursor.next_if(Punctuator::CloseParen, interner)?.is_some() {
                        break;
                    }
                }
                _ => {
                    if !args.is_empty() {
                        return Err(Error::expected(
                            [",".to_owned(), "}".to_owned()],
                            next_token.to_string(interner),
                            next_token.span(),
                            "argument list",
                        ));
                    }
                }
            }

            if cursor.next_if(Punctuator::Spread, interner)?.is_some() {
                args.push(
                    Spread::new(
                        AssignmentExpression::new(None, true, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?,
                    )
                    .into(),
                );
            } else {
                args.push(
                    AssignmentExpression::new(None, true, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?,
                );
            }
        }
        cursor.set_goal(InputElement::Div);
        Ok(args.into_boxed_slice())
    }
}
