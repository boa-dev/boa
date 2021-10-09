use crate::{
    syntax::{
        ast::{node::WhileLoop, Keyword, Node, Punctuator},
        parser::{expression::Expression, statement::Statement, Cursor, ParseError, TokenParser},
    },
    BoaProfiler,
};

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
pub(in crate::syntax::parser::statement) struct WhileStatement<
    const YIELD: bool,
    const AWAIT: bool,
    const RETURN: bool,
>;

impl<R, const YIELD: bool, const AWAIT: bool, const RETURN: bool> TokenParser<R>
    for WhileStatement<YIELD, AWAIT, RETURN>
where
    R: Read,
{
    type Output = WhileLoop;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("WhileStatement", "Parsing");
        cursor.expect(Keyword::While, "while statement")?;

        cursor.expect(Punctuator::OpenParen, "while statement")?;

        let cond = Expression::<true, YIELD, AWAIT>.parse(cursor)?;

        let position = cursor
            .expect(Punctuator::CloseParen, "while statement")?
            .span()
            .end();

        let body = Statement::<YIELD, AWAIT, RETURN>.parse(cursor)?;

        // Early Error: It is a Syntax Error if IsLabelledFunction(Statement) is true.
        if let Node::FunctionDecl(_) = body {
            return Err(ParseError::wrong_function_declaration_non_strict(position));
        }

        Ok(WhileLoop::new(cond, body))
    }
}
