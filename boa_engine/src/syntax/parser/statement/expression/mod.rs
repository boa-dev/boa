use crate::syntax::{
    ast::{node::Node, Keyword, Punctuator},
    lexer::TokenKind,
    parser::{
        expression::Expression, AllowAwait, AllowYield, Cursor, ParseError, ParseResult,
        TokenParser,
    },
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// Expression statement parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ExpressionStatement
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser::statement) struct ExpressionStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ExpressionStatement {
    /// Creates a new `ExpressionStatement` parser.
    pub(in crate::syntax::parser::statement) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for ExpressionStatement
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        let _timer = Profiler::global().start_event("ExpressionStatement", "Parsing");

        let next_token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
        match next_token.kind() {
            TokenKind::Keyword(Keyword::Function | Keyword::Class) => {
                return Err(ParseError::general(
                    "expected statement",
                    next_token.span().start(),
                ));
            }
            TokenKind::Keyword(Keyword::Async) => {
                let next_token = cursor.peek(1, interner)?.ok_or(ParseError::AbruptEnd)?;
                if next_token.kind() == &TokenKind::Keyword(Keyword::Function) {
                    return Err(ParseError::general(
                        "expected statement",
                        next_token.span().start(),
                    ));
                }
            }
            TokenKind::Keyword(Keyword::Let) => {
                let next_token = cursor.peek(1, interner)?.ok_or(ParseError::AbruptEnd)?;
                if next_token.kind() == &TokenKind::Punctuator(Punctuator::OpenBracket) {
                    return Err(ParseError::general(
                        "expected statement",
                        next_token.span().start(),
                    ));
                }
            }
            _ => {}
        }

        let expr = Expression::new(None, true, self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;

        cursor.expect_semicolon("expression statement", interner)?;

        Ok(expr)
    }
}
