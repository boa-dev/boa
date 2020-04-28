//! For statement parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for
//! [spec]: https://tc39.es/ecma262/#sec-for-statement

use crate::syntax::{
    ast::{keyword::Keyword, node::Node, punc::Punctuator, token::TokenKind},
    parser::{
        expression::Expression,
        statement::declaration::Declaration,
        statement::{variable::VariableDeclarationList, Statement},
        AllowAwait, AllowReturn, AllowYield, Cursor, ParseError, ParseResult, TokenParser,
    },
};

/// For statement parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for
/// [spec]: https://tc39.es/ecma262/#sec-for-statement
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser::statement) struct ForStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl ForStatement {
    /// Creates a new `ForStatement` parser.
    pub(in crate::syntax::parser::statement) fn new<Y, A, R>(
        allow_yield: Y,
        allow_await: A,
        allow_return: R,
    ) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        R: Into<AllowReturn>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            allow_return: allow_return.into(),
        }
    }
}

impl TokenParser for ForStatement {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        cursor.expect(Keyword::For, "for statement")?;
        cursor.expect(Punctuator::OpenParen, "for statement")?;

        let init = match cursor.peek(0).ok_or(ParseError::AbruptEnd)?.kind {
            TokenKind::Keyword(Keyword::Var) => Some(
                VariableDeclarationList::new(false, self.allow_yield, self.allow_await)
                    .parse(cursor)?,
            ),
            TokenKind::Keyword(Keyword::Let) | TokenKind::Keyword(Keyword::Const) => {
                Some(Declaration::new(self.allow_yield, self.allow_await).parse(cursor)?)
            }
            TokenKind::Punctuator(Punctuator::Semicolon) => None,
            _ => Some(Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?),
        };

        cursor.expect(Punctuator::Semicolon, "for statement")?;

        let cond = if cursor.next_if(Punctuator::Semicolon).is_some() {
            Node::const_node(true)
        } else {
            let step = Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;
            cursor.expect(Punctuator::Semicolon, "for statement")?;
            step
        };

        let step = if cursor.next_if(Punctuator::CloseParen).is_some() {
            None
        } else {
            let step = Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;
            cursor.expect(
                TokenKind::Punctuator(Punctuator::CloseParen),
                "for statement",
            )?;
            Some(step)
        };

        let body =
            Statement::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor)?;

        let for_node = Node::for_loop::<_, _, _, Node, Node, Node, _>(init, cond, step, body);

        Ok(Node::Block(vec![for_node]))
    }
}
