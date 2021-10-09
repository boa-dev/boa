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
        parser::{cursor::SemicolonResult, Cursor, ParseResult, TokenParser},
    },
    BoaProfiler,
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
pub(in crate::syntax::parser) struct YieldExpression<const IN: bool, const AWAIT: bool>;

impl<R, const IN: bool, const AWAIT: bool> TokenParser<R> for YieldExpression<IN, AWAIT>
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("YieldExpression", "Parsing");

        cursor.expect(TokenKind::Keyword(Keyword::Yield), "yield expression")?;

        let mut expr = None;
        let mut delegate = false;

        if let SemicolonResult::Found(_) = cursor.peek_semicolon()? {
            cursor.expect(
                TokenKind::Punctuator(Punctuator::Semicolon),
                "token disappeared",
            )?;
        } else if let Ok(next_token) = cursor.peek_expect_no_lineterminator(0, "yield expression") {
            if let TokenKind::Punctuator(Punctuator::Mul) = next_token.kind() {
                cursor.expect(TokenKind::Punctuator(Punctuator::Mul), "token disappeared")?;
                delegate = true;
                expr = Some(AssignmentExpression::<IN, true, AWAIT>.parse(cursor)?);
            } else {
                expr = Some(AssignmentExpression::<IN, true, AWAIT>.parse(cursor)?);
            }
        }

        Ok(Node::Yield(Yield::new::<Node, Option<Node>>(
            expr, delegate,
        )))
    }
}
