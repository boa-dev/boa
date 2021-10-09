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
        parser::{expression::AssignmentExpression, Cursor, ParseError, TokenParser},
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
pub(in crate::syntax::parser::expression) struct Arguments<const YIELD: bool, const AWAIT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for Arguments<YIELD, AWAIT>
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
                    Spread::new(AssignmentExpression::<true, YIELD, AWAIT>.parse(cursor)?).into(),
                );
            } else {
                cursor.set_goal(InputElement::RegExp);
                args.push(AssignmentExpression::<true, YIELD, AWAIT>.parse(cursor)?);
            }
        }
        Ok(args.into_boxed_slice())
    }
}
