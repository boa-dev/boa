//! Hoistable declaration parsing.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#prod-HoistableDeclaration

use crate::syntax::{
    ast::{keyword::Keyword, node::Node, punc::Punctuator},
    parser::{
        function::FormalParameters, function::FunctionBody, statement::BindingIdentifier,
        AllowAwait, AllowDefault, AllowYield, Cursor, ParseResult, TokenParser,
    },
};

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
    allow_default: AllowDefault,
}

impl HoistableDeclaration {
    /// Creates a new `HoistableDeclaration` parser.
    pub(super) fn new<Y, A, D>(allow_yield: Y, allow_await: A, allow_default: D) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        D: Into<AllowDefault>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            allow_default: allow_default.into(),
        }
    }
}

impl TokenParser for HoistableDeclaration {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        // TODO: check for generators and async functions + generators
        FunctionDeclaration::new(self.allow_yield, self.allow_await, self.allow_default)
            .parse(cursor)
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
    allow_default: AllowDefault,
}

impl FunctionDeclaration {
    /// Creates a new `FunctionDeclaration` parser.
    fn new<Y, A, D>(allow_yield: Y, allow_await: A, allow_default: D) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        D: Into<AllowDefault>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            allow_default: allow_default.into(),
        }
    }
}

impl TokenParser for FunctionDeclaration {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        cursor.expect(Keyword::Function, "function declaration")?;

        let name = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;

        cursor.expect(Punctuator::OpenParen, "function declaration")?;

        let params = FormalParameters::new(false, false).parse(cursor)?;

        cursor.expect(Punctuator::CloseParen, "function declaration")?;
        cursor.expect(Punctuator::OpenBlock, "function declaration")?;

        let body = FunctionBody::new(self.allow_yield, self.allow_await)
            .parse(cursor)
            .map(Node::statement_list)?;

        cursor.expect(Punctuator::CloseBlock, "function declaration")?;

        Ok(Node::function_decl(name, params, body))
    }
}
