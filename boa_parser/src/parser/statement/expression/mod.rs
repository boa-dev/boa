use crate::{
    lexer::TokenKind,
    parser::{
        expression::Expression, AllowAwait, AllowYield, Cursor, OrAbrupt, ParseResult, TokenParser,
    },
    Error,
};
use boa_ast::{Keyword, Punctuator, Statement};
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
pub(in crate::parser::statement) struct ExpressionStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ExpressionStatement {
    /// Creates a new `ExpressionStatement` parser.
    pub(in crate::parser::statement) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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
    type Output = Statement;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("ExpressionStatement", "Parsing");

        let next_token = cursor.peek(0, interner).or_abrupt()?;
        match next_token.kind() {
            TokenKind::Keyword((
                Keyword::Function | Keyword::Class | Keyword::Async | Keyword::Let,
                true,
            )) => {
                return Err(Error::general(
                    "Keyword must not contain escaped characters",
                    next_token.span().start(),
                ));
            }
            TokenKind::Keyword((Keyword::Function | Keyword::Class, false)) => {
                return Err(Error::general(
                    "expected statement",
                    next_token.span().start(),
                ));
            }
            TokenKind::Keyword((Keyword::Async, false)) => {
                let next_token = cursor.peek(1, interner).or_abrupt()?;
                match next_token.kind() {
                    TokenKind::Keyword((Keyword::Function, true)) => {
                        return Err(Error::general(
                            "Keyword must not contain escaped characters",
                            next_token.span().start(),
                        ));
                    }
                    TokenKind::Keyword((Keyword::Function, false)) => {
                        return Err(Error::general(
                            "expected statement",
                            next_token.span().start(),
                        ));
                    }
                    _ => {}
                }
            }
            TokenKind::Keyword((Keyword::Let, false)) => {
                let next_token = cursor.peek(1, interner).or_abrupt()?;
                if next_token.kind() == &TokenKind::Punctuator(Punctuator::OpenBracket) {
                    return Err(Error::general(
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

        Ok(expr.into())
    }
}
