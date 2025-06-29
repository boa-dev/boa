//! Declaration parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements#Declarations
//! [spec]:https://tc39.es/ecma262/#sec-declarations-and-the-variable-statement

mod export;
mod hoistable;
mod import;
mod lexical;
#[cfg(test)]
mod tests;

pub(in crate::parser) use self::{
    export::ExportDeclaration,
    hoistable::{
        class_decl::ClassTail, ClassDeclaration, FunctionDeclaration, HoistableDeclaration,
    },
    import::ImportDeclaration,
    lexical::{allowed_token_after_let, LexicalDeclaration},
};
use crate::{
    lexer::TokenKind, parse_cmd, parser::{parse_loop::ParseState, AllowAwait, AllowYield, ControlFlow, Cursor, OrAbrupt, ParseResult, TokenLoopParser, TokenParser}, source::ReadChar, Error
};
use boa_ast::{self as ast, Keyword};
use boa_interner::{Interner, Sym};

/// Parses a declaration.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-Declaration
#[derive(Debug, Clone, Copy)]
pub(super) struct Declaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl Declaration {
    /// Creates a new declaration parser.
    #[inline]
    pub(super) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for Declaration
where
    R: ReadChar,
{
    type Output = ast::Declaration;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let tok = cursor.peek(0, interner).or_abrupt()?;

        match tok.kind() {
            TokenKind::Keyword((Keyword::Function | Keyword::Async | Keyword::Class, _)) => {
                HoistableDeclaration::new(self.allow_yield, self.allow_await, false)
                    .parse(cursor, interner)
            }
            TokenKind::Keyword((Keyword::Const | Keyword::Let, _)) => {
                LexicalDeclaration::new(true, self.allow_yield, self.allow_await, false)
                    .parse(cursor, interner)
                    .map(Into::into)
            }
            _ => Err(Error::expected(
                [
                    Keyword::Function.to_string(),
                    Keyword::Async.to_string(),
                    Keyword::Class.to_string(),
                    Keyword::Const.to_string(),
                    Keyword::Let.to_string(),
                ],
                tok.to_string(interner),
                tok.span(),
                "export declaration",
            )),
        }
    }
}

impl<R: ReadChar> TokenLoopParser<R> for Declaration {
    fn parse_loop(&mut self, state: &mut ParseState<'_, R>, continue_point: usize) -> ParseResult<ControlFlow<R>> {
        if continue_point == 1 {
            parse_cmd![[POP LOCAL]: state => Empty];
            parse_cmd![[PEEK NODE]: state match Declaration]; // pass value up
            return Ok(ControlFlow::Done)
        } else if continue_point > 1 {
            return state.continue_point_error(continue_point)
        }

        let tok = state.cursor.peek(0, state.interner).or_abrupt()?;

        match tok.kind() {
            TokenKind::Keyword((Keyword::Function | Keyword::Async | Keyword::Class, _)) => {
                let node = HoistableDeclaration::new(self.allow_yield, self.allow_await, false);
                parse_cmd![[SUB PARSE]: node; state <= Empty (1)];
            }
            TokenKind::Keyword((Keyword::Const | Keyword::Let, _)) => {
                let node = LexicalDeclaration::new(true, self.allow_yield, self.allow_await, false);
                parse_cmd![[SUB PARSE]: node; state <= Empty (1)];
            }
            _ => return Err(Error::expected(
                [
                    Keyword::Function.to_string(),
                    Keyword::Async.to_string(),
                    Keyword::Class.to_string(),
                    Keyword::Const.to_string(),
                    Keyword::Let.to_string(),
                ],
                tok.to_string(state.interner),
                tok.span(),
                "export declaration",
            )),
        }
    }
}

/// Parses a `from` clause.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FromClause
#[derive(Debug, Clone, Copy)]
struct FromClause {
    context: &'static str,
}

impl FromClause {
    /// Creates a new `from` clause parser
    #[inline]
    const fn new(context: &'static str) -> Self {
        Self { context }
    }
}

impl<R> TokenParser<R> for FromClause
where
    R: ReadChar,
{
    type Output = ast::declaration::ModuleSpecifier;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        cursor.expect(TokenKind::identifier(Sym::FROM), self.context, interner)?;

        let tok = cursor.next(interner).or_abrupt()?;

        let TokenKind::StringLiteral((from, _)) = tok.kind() else {
            return Err(Error::expected(
                ["string literal".to_owned()],
                tok.to_string(interner),
                tok.span(),
                self.context,
            ));
        };

        Ok((*from).into())
    }
}
