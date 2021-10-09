//! Do-while statement parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/do...while
//! [spec]: https://tc39.es/ecma262/#sec-do-while-statement

use crate::{
    syntax::{
        ast::{node::DoWhileLoop, Keyword, Node, Punctuator},
        lexer::TokenKind,
        parser::{expression::Expression, statement::Statement, Cursor, ParseError, TokenParser},
    },
    BoaProfiler,
};
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
pub(in crate::syntax::parser::statement) struct DoWhileStatement<
    const YIELD: bool,
    const AWAIT: bool,
    const RETURN: bool,
>;

impl<R, const YIELD: bool, const AWAIT: bool, const RETURN: bool> TokenParser<R>
    for DoWhileStatement<YIELD, AWAIT, RETURN>
where
    R: Read,
{
    type Output = DoWhileLoop;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("DoWhileStatement", "Parsing");

        let position = cursor
            .expect(Keyword::Do, "do while statement")?
            .span()
            .end();

        let body = Statement::<YIELD, AWAIT, RETURN>.parse(cursor)?;

        // Early Error: It is a Syntax Error if IsLabelledFunction(Statement) is true.
        if let Node::FunctionDecl(_) = body {
            return Err(ParseError::wrong_function_declaration_non_strict(position));
        }

        let next_token = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

        if next_token.kind() != &TokenKind::Keyword(Keyword::While) {
            return Err(ParseError::expected(
                vec![TokenKind::Keyword(Keyword::While)],
                next_token.clone(),
                "do while statement",
            ));
        }

        cursor.expect(Keyword::While, "do while statement")?;

        cursor.expect(Punctuator::OpenParen, "do while statement")?;

        let cond = Expression::<true, YIELD, AWAIT>.parse(cursor)?;

        cursor.expect(Punctuator::CloseParen, "do while statement")?;

        // Here, we only care to read the next token if it's a semicolon. If it's not, we
        // automatically "enter" or assume a semicolon, since we have just read the `)` token:
        // https://tc39.es/ecma262/#sec-automatic-semicolon-insertion
        if let Some(tok) = cursor.peek(0)? {
            if let TokenKind::Punctuator(Punctuator::Semicolon) = *tok.kind() {
                cursor.next()?;
            }
        }

        Ok(DoWhileLoop::new(body, cond))
    }
}
