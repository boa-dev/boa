use crate::syntax::{
    ast::{node::WhileLoop, Keyword, Node, Punctuator},
    parser::{
        expression::Expression, statement::Statement, AllowAwait, AllowReturn, AllowYield, Cursor,
        ParseError, TokenParser,
    },
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// While statement parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/while
/// [spec]: https://tc39.es/ecma262/#sec-while-statement
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser::statement) struct WhileStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl WhileStatement {
    /// Creates a new `WhileStatement` parser.
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

impl<R> TokenParser<R> for WhileStatement
where
    R: Read,
{
    type Output = WhileLoop;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("WhileStatement", "Parsing");
        cursor.expect((Keyword::While, false), "while statement", interner)?;

        cursor.expect(Punctuator::OpenParen, "while statement", interner)?;

        let cond = Expression::new(None, true, self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;

        let position = cursor
            .expect(Punctuator::CloseParen, "while statement", interner)?
            .span()
            .end();

        let body = Statement::new(self.allow_yield, self.allow_await, self.allow_return)
            .parse(cursor, interner)?;

        // Early Error: It is a Syntax Error if IsLabelledFunction(Statement) is true.
        if let Node::FunctionDecl(_) = body {
            return Err(ParseError::wrong_function_declaration_non_strict(position));
        }

        Ok(WhileLoop::new(cond, body))
    }
}
