use super::super::{expression::Expression, ParseResult};
use crate::{
    syntax::{
        ast::{node::Node, Keyword, Punctuator},
        lexer::TokenKind,
        parser::{Cursor, ParseError, TokenParser},
    },
    BoaProfiler,
};
use std::io::Read;

/// Expression statement parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ExpressionStatement
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser::statement) struct ExpressionStatement<
    const YIELD: bool,
    const AWAIT: bool,
>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for ExpressionStatement<YIELD, AWAIT>
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("ExpressionStatement", "Parsing");

        let next_token = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;
        match next_token.kind() {
            TokenKind::Keyword(Keyword::Function) | TokenKind::Keyword(Keyword::Class) => {
                return Err(ParseError::general(
                    "expected statement",
                    next_token.span().start(),
                ));
            }
            TokenKind::Keyword(Keyword::Async) => {
                let next_token = cursor.peek(1)?.ok_or(ParseError::AbruptEnd)?;
                if next_token.kind() == &TokenKind::Keyword(Keyword::Function) {
                    return Err(ParseError::general(
                        "expected statement",
                        next_token.span().start(),
                    ));
                }
            }
            TokenKind::Keyword(Keyword::Let) => {
                let next_token = cursor.peek(1)?.ok_or(ParseError::AbruptEnd)?;
                if next_token.kind() == &TokenKind::Punctuator(Punctuator::OpenBracket) {
                    return Err(ParseError::general(
                        "expected statement",
                        next_token.span().start(),
                    ));
                }
            }
            _ => {}
        }

        let expr = Expression::<true, YIELD, AWAIT>.parse(cursor)?;

        cursor.expect_semicolon("expression statement")?;

        Ok(expr)
    }
}
