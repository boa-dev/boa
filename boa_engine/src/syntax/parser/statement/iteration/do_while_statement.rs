//! Do-while statement parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/do...while
//! [spec]: https://tc39.es/ecma262/#sec-do-while-statement

use crate::syntax::{
    ast::{node::DoWhileLoop, Keyword, Node, Punctuator},
    lexer::TokenKind,
    parser::{
        expression::Expression, statement::Statement, AllowAwait, AllowReturn, AllowYield, Cursor,
        ParseError, TokenParser,
    },
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// Do...while statement parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/do...while
/// [spec]: https://tc39.es/ecma262/#sec-do-while-statement
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser::statement) struct DoWhileStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl DoWhileStatement {
    /// Creates a new `DoWhileStatement` parser.
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

impl<R> TokenParser<R> for DoWhileStatement
where
    R: Read,
{
    type Output = DoWhileLoop;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("DoWhileStatement", "Parsing");

        let position = cursor
            .expect(Keyword::Do, "do while statement", interner)?
            .span()
            .end();

        let body = Statement::new(self.allow_yield, self.allow_await, self.allow_return)
            .parse(cursor, interner)?;

        // Early Error: It is a Syntax Error if IsLabelledFunction(Statement) is true.
        if let Node::FunctionDecl(_) = body {
            return Err(ParseError::wrong_function_declaration_non_strict(position));
        }

        let next_token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;

        if next_token.kind() != &TokenKind::Keyword(Keyword::While) {
            return Err(ParseError::expected(
                ["while".to_owned()],
                next_token.to_string(interner),
                next_token.span(),
                "do while statement",
            ));
        }

        cursor.expect(Keyword::While, "do while statement", interner)?;

        cursor.expect(Punctuator::OpenParen, "do while statement", interner)?;

        let cond = Expression::new(None, true, self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;

        cursor.expect(Punctuator::CloseParen, "do while statement", interner)?;

        // Here, we only care to read the next token if it's a semicolon. If it's not, we
        // automatically "enter" or assume a semicolon, since we have just read the `)` token:
        // https://tc39.es/ecma262/#sec-automatic-semicolon-insertion
        if let Some(tok) = cursor.peek(0, interner)? {
            if let TokenKind::Punctuator(Punctuator::Semicolon) = *tok.kind() {
                cursor.next(interner)?;
            }
        }

        Ok(DoWhileLoop::new(body, cond))
    }
}
