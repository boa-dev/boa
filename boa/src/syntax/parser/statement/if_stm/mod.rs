#[cfg(test)]
mod tests;

use crate::{
    syntax::{
        ast::{node::If, Keyword, Node, Punctuator},
        lexer::TokenKind,
        parser::{
            expression::Expression,
            statement::{declaration::hoistable::FunctionDeclaration, Statement},
            AllowAwait, AllowReturn, AllowYield, Cursor, ParseError, TokenParser,
        },
    },
    BoaProfiler, Interner,
};
use std::io::Read;

/// If statement parsing.
///
/// An _If_ statement will have a condition, a block statemet, and an optional _else_ statement.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/if...else
/// [spec]: https://tc39.es/ecma262/#prod-IfStatement
#[derive(Debug, Clone, Copy)]
pub(super) struct IfStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl IfStatement {
    /// Creates a new `IfStatement` parser.
    pub(super) fn new<Y, A, R>(allow_yield: Y, allow_await: A, allow_return: R) -> Self
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

impl<R> TokenParser<R> for IfStatement
where
    R: Read,
{
    type Output = If;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("IfStatement", "Parsing");

        cursor.expect(Keyword::If, "if statement", interner)?;
        cursor.expect(Punctuator::OpenParen, "if statement", interner)?;

        let condition = Expression::new(None, true, self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;

        let position = cursor
            .expect(Punctuator::CloseParen, "if statement", interner)?
            .span()
            .end();

        let then_node = if !cursor.strict_mode()
            && cursor
                .peek(0, interner)?
                .ok_or(ParseError::AbruptEnd)?
                .kind()
                == &TokenKind::Keyword(Keyword::Function)
        {
            // FunctionDeclarations in IfStatement Statement Clauses
            // https://tc39.es/ecma262/#sec-functiondeclarations-in-ifstatement-statement-clauses
            FunctionDeclaration::new(self.allow_yield, self.allow_await, false)
                .parse(cursor, interner)?
                .into()
        } else {
            let node = Statement::new(self.allow_yield, self.allow_await, self.allow_return)
                .parse(cursor, interner)?;

            // Early Error: It is a Syntax Error if IsLabelledFunction(the first Statement) is true.
            if let Node::FunctionDecl(_) = node {
                return Err(ParseError::wrong_function_declaration_non_strict(position));
            }

            node
        };

        let else_node = if cursor.next_if(Keyword::Else, interner)?.is_some() {
            let position = cursor
                .peek(0, interner)?
                .ok_or(ParseError::AbruptEnd)?
                .span()
                .start();

            if !cursor.strict_mode()
                && cursor
                    .peek(0, interner)?
                    .ok_or(ParseError::AbruptEnd)?
                    .kind()
                    == &TokenKind::Keyword(Keyword::Function)
            {
                // FunctionDeclarations in IfStatement Statement Clauses
                // https://tc39.es/ecma262/#sec-functiondeclarations-in-ifstatement-statement-clauses
                Some(
                    FunctionDeclaration::new(self.allow_yield, self.allow_await, false)
                        .parse(cursor, interner)?
                        .into(),
                )
            } else {
                let node = Statement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor, interner)?;

                // Early Error: It is a Syntax Error if IsLabelledFunction(the second Statement) is true.
                if let Node::FunctionDecl(_) = node {
                    return Err(ParseError::wrong_function_declaration_non_strict(position));
                }

                Some(node)
            }
        } else {
            None
        };

        Ok(If::new::<_, _, Node, _>(condition, then_node, else_node))
    }
}
