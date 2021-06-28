//! For statement parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for
//! [spec]: https://tc39.es/ecma262/#sec-for-statement

use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{
            node::{ForInLoop, ForLoop, ForOfLoop, Node},
            Const, Keyword, Punctuator,
        },
        parser::{
            expression::Expression,
            statement::declaration::Declaration,
            statement::{variable::VariableDeclarationList, Statement},
            AllowAwait, AllowReturn, AllowYield, Cursor, ParseError, TokenParser,
        },
    },
    BoaProfiler,
};

use std::io::Read;

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

impl<R> TokenParser<R> for ForStatement
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ForStatement", "Parsing");
        cursor.expect(Keyword::For, "for statement")?;
        cursor.expect(Punctuator::OpenParen, "for statement")?;

        let init = match cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind() {
            TokenKind::Keyword(Keyword::Var) => {
                let _ = cursor.next()?;
                Some(
                    VariableDeclarationList::new(false, self.allow_yield, self.allow_await)
                        .parse(cursor)
                        .map(|(kind, span)| Node::new(kind, span))?,
                )
            }
            TokenKind::Keyword(Keyword::Let) | TokenKind::Keyword(Keyword::Const) => {
                Some(Declaration::new(self.allow_yield, self.allow_await, false).parse(cursor)?)
            }
            TokenKind::Punctuator(Punctuator::Semicolon) => None,
            _ => Some(Expression::new(false, self.allow_yield, self.allow_await).parse(cursor)?),
        };

        match cursor.peek(0)? {
            Some(tok) if tok.kind() == &TokenKind::Keyword(Keyword::In) && init.is_some() => {
                let _ = cursor.next();
                let expr =
                    Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;
                cursor.expect(Punctuator::CloseParen, "for in statement")?;
                let body = Statement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)?;
                return Ok(ForInLoop::new(init.unwrap(), expr, body).into());
            }
            Some(tok) if tok.kind() == &TokenKind::Keyword(Keyword::Of) && init.is_some() => {
                let _ = cursor.next();
                let iterable =
                    Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;
                cursor.expect(Punctuator::CloseParen, "for of statement")?;
                let body = Statement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)?;
                return Ok(ForOfLoop::new(init.unwrap(), iterable, body).into());
            }
            _ => {}
        }

        cursor.expect(Punctuator::Semicolon, "for statement")?;

        let cond = if cursor.next_if(Punctuator::Semicolon)?.is_some() {
            Const::from(true).into()
        } else {
            let step = Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;
            cursor.expect(Punctuator::Semicolon, "for statement")?;
            step
        };

        let step = if cursor.next_if(Punctuator::CloseParen)?.is_some() {
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

        // TODO: do not encapsulate the `for` in a block just to have an inner scope.
        Ok(ForLoop::new(init, cond, step, body).into())
    }
}
