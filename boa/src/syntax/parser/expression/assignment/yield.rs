//! YieldExpression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/yield
//! [spec]: https://tc39.es/ecma262/#prod-YieldExpression

use crate::{
    syntax::{
        ast::{
            node::{Node, Yield},
            Keyword, Punctuator,
        },
        lexer::TokenKind,
        parser::{cursor::SemicolonResult, AllowAwait, AllowIn, Cursor, ParseResult, TokenParser},
    },
    BoaProfiler, Interner,
};

use std::io::Read;

use super::AssignmentExpression;

/// YieldExpression parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/yield
/// [spec]: https://tc39.es/ecma262/#prod-YieldExpression
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct YieldExpression {
    allow_in: AllowIn,
    allow_await: AllowAwait,
}

impl YieldExpression {
    /// Creates a new `YieldExpression` parser.
    pub(in crate::syntax::parser) fn new<I, A>(allow_in: I, allow_await: A) -> Self
    where
        I: Into<AllowIn>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for YieldExpression
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("YieldExpression", "Parsing");

        cursor.expect(
            TokenKind::Keyword(Keyword::Yield),
            "yield expression",
            interner,
        )?;

        let mut expr = None;
        let mut delegate = false;

        if let SemicolonResult::Found(_) = cursor.peek_semicolon(interner)? {
            cursor.expect(
                TokenKind::Punctuator(Punctuator::Semicolon),
                "token disappeared",
                interner,
            )?;
        } else if let Ok(next_token) =
            cursor.peek_expect_no_lineterminator(0, "yield expression", interner)
        {
            if let TokenKind::Punctuator(Punctuator::Mul) = next_token.kind() {
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::Mul),
                    "token disappeared",
                    interner,
                )?;
                delegate = true;
            }
            expr = Some(
                AssignmentExpression::new(self.allow_in, true, self.allow_await)
                    .parse(cursor, interner)?,
            );
        }

        Ok(Node::Yield(Yield::new::<Node, Option<Node>>(
            expr, delegate,
        )))
    }
}
