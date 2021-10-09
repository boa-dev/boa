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
            Cursor, ParseError, TokenParser,
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
pub(in crate::syntax::parser::statement) struct ForStatement<
    const YIELD: bool,
    const AWAIT: bool,
    const RETURN: bool,
>;

impl<R, const YIELD: bool, const AWAIT: bool, const RETURN: bool> TokenParser<R>
    for ForStatement<YIELD, AWAIT, RETURN>
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
                    VariableDeclarationList::<false, YIELD, AWAIT>
                        .parse(cursor)
                        .map(Node::from)?,
                )
            }
            TokenKind::Keyword(Keyword::Let) | TokenKind::Keyword(Keyword::Const) => {
                Some(Declaration::<YIELD, AWAIT, false>.parse(cursor)?)
            }
            TokenKind::Punctuator(Punctuator::Semicolon) => None,
            _ => Some(Expression::<false, YIELD, AWAIT>.parse(cursor)?),
        };

        match cursor.peek(0)? {
            Some(tok) if tok.kind() == &TokenKind::Keyword(Keyword::In) && init.is_some() => {
                let _ = cursor.next();
                let expr = Expression::<true, YIELD, AWAIT>.parse(cursor)?;

                let position = cursor
                    .expect(Punctuator::CloseParen, "for in statement")?
                    .span()
                    .end();

                let body = Statement::<YIELD, AWAIT, RETURN>.parse(cursor)?;

                // Early Error: It is a Syntax Error if IsLabelledFunction(the first Statement) is true.
                if let Node::FunctionDecl(_) = body {
                    return Err(ParseError::wrong_function_declaration_non_strict(position));
                }

                return Ok(ForInLoop::new(init.unwrap(), expr, body).into());
            }
            Some(tok) if tok.kind() == &TokenKind::Keyword(Keyword::Of) && init.is_some() => {
                let _ = cursor.next();
                let iterable = Expression::<true, YIELD, AWAIT>.parse(cursor)?;

                let position = cursor
                    .expect(Punctuator::CloseParen, "for of statement")?
                    .span()
                    .end();

                let body = Statement::<YIELD, AWAIT, RETURN>.parse(cursor)?;

                // Early Error: It is a Syntax Error if IsLabelledFunction(the first Statement) is true.
                if let Node::FunctionDecl(_) = body {
                    return Err(ParseError::wrong_function_declaration_non_strict(position));
                }

                return Ok(ForOfLoop::new(init.unwrap(), iterable, body).into());
            }
            _ => {}
        }

        cursor.expect(Punctuator::Semicolon, "for statement")?;

        let cond = if cursor.next_if(Punctuator::Semicolon)?.is_some() {
            Const::from(true).into()
        } else {
            let step = Expression::<true, YIELD, AWAIT>.parse(cursor)?;
            cursor.expect(Punctuator::Semicolon, "for statement")?;
            step
        };

        let step = if cursor.next_if(Punctuator::CloseParen)?.is_some() {
            None
        } else {
            let step = Expression::<true, YIELD, AWAIT>.parse(cursor)?;
            cursor.expect(
                TokenKind::Punctuator(Punctuator::CloseParen),
                "for statement",
            )?;
            Some(step)
        };

        let position = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.span().start();

        let body = Statement::<YIELD, AWAIT, RETURN>.parse(cursor)?;

        // Early Error: It is a Syntax Error if IsLabelledFunction(the first Statement) is true.
        if let Node::FunctionDecl(_) = body {
            return Err(ParseError::wrong_function_declaration_non_strict(position));
        }

        // TODO: do not encapsulate the `for` in a block just to have an inner scope.
        Ok(ForLoop::new(init, cond, step, body).into())
    }
}
