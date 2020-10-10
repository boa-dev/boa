//! Hoistable declaration parsing.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#prod-HoistableDeclaration

use crate::{
    syntax::{
        ast::{
            node::{AsyncFunctionDecl, FunctionDecl},
            Keyword, Node, Punctuator,
        },
        lexer::TokenKind,
        parser::{
            function::FormalParameters, function::FunctionBody, statement::BindingIdentifier,
            AllowAwait, AllowDefault, AllowYield, Cursor, ParseError, ParseResult, TokenParser,
        },
    },
    BoaProfiler,
};
use std::io::Read;

/// Hoistable declaration parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FunctionDeclaration
#[derive(Debug, Clone, Copy)]
pub(super) struct HoistableDeclaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_default: AllowDefault,
}

impl HoistableDeclaration {
    /// Creates a new `HoistableDeclaration` parser.
    pub(super) fn new<Y, A, D>(allow_yield: Y, allow_await: A, is_default: D) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        D: Into<AllowDefault>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            is_default: is_default.into(),
        }
    }
}

impl<R> TokenParser<R> for HoistableDeclaration
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("HoistableDeclaration", "Parsing");
        let tok = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

        match tok.kind() {
            TokenKind::Keyword(Keyword::Function) => {
                FunctionDeclaration::new(self.allow_yield, self.allow_await, self.is_default)
                    .parse(cursor)
                    .map(Node::from)
            }
            TokenKind::Keyword(Keyword::Async) => {
                AsyncFunctionDeclaration::new(self.allow_yield, self.allow_await, false)
                    .parse(cursor)
                    .map(Node::from)
            }
            _ => unreachable!("unknown token found: {:?}", tok),
        }
    }
}

/// Function declaration parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function
/// [spec]: https://tc39.es/ecma262/#prod-FunctionDeclaration
#[derive(Debug, Clone, Copy)]
struct FunctionDeclaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_default: AllowDefault,
}

impl FunctionDeclaration {
    /// Creates a new `FunctionDeclaration` parser.
    fn new<Y, A, D>(allow_yield: Y, allow_await: A, is_default: D) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        D: Into<AllowDefault>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            is_default: is_default.into(),
        }
    }
}

impl<R> TokenParser<R> for FunctionDeclaration
where
    R: Read,
{
    type Output = FunctionDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Function, "function declaration")?;

        // TODO: If self.is_default, then this can be empty.
        let name = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;

        cursor.expect(Punctuator::OpenParen, "function declaration")?;

        let params = FormalParameters::new(false, false).parse(cursor)?;

        cursor.expect(Punctuator::CloseParen, "function declaration")?;
        cursor.expect(Punctuator::OpenBlock, "function declaration")?;

        let body = FunctionBody::new(self.allow_yield, self.allow_await).parse(cursor)?;

        cursor.expect(Punctuator::CloseBlock, "function declaration")?;

        Ok(FunctionDecl::new(name, params, body))
    }
}

/// Async Function declaration parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]:
/// [spec]:
#[derive(Debug, Clone, Copy)]
struct AsyncFunctionDeclaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_default: AllowDefault,
}

impl AsyncFunctionDeclaration {
    /// Creates a new `FunctionDeclaration` parser.
    fn new<Y, A, D>(allow_yield: Y, allow_await: A, is_default: D) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        D: Into<AllowDefault>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            is_default: is_default.into(),
        }
    }
}

impl<R> TokenParser<R> for AsyncFunctionDeclaration
where
    R: Read,
{
    type Output = AsyncFunctionDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Async, "async function declaration")?;
        let tok = cursor.peek_expect_no_lineterminator(0)?;

        match tok.kind() {
            TokenKind::Keyword(Keyword::Function) => {}
            _ => {}
        }

        unimplemented!("AsyncFunctionDecl parse");
    }
}
