#[cfg(test)]
mod tests;

use crate::{
    lexer::TokenKind,
    parser::{
        expression::Expression, statement::declaration::FunctionDeclaration, AllowAwait,
        AllowReturn, AllowYield, Cursor, OrAbrupt, ParseResult, TokenParser,
    },
    Error,
};
use boa_ast::{
    statement::{Block, If},
    Declaration, Keyword, Punctuator, StatementListItem,
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

use super::Statement;

/// If statement parsing.
///
/// An `if` statement will have a condition, a block statement, and an optional `else` statement.
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

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("IfStatement", "Parsing");

        cursor.expect((Keyword::If, false), "if statement", interner)?;
        cursor.expect(Punctuator::OpenParen, "if statement", interner)?;

        let condition = Expression::new(None, true, self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;

        let position = cursor
            .expect(Punctuator::CloseParen, "if statement", interner)?
            .span()
            .end();

        let strict = cursor.strict_mode();
        let token = cursor.peek(0, interner).or_abrupt()?;
        let then_node = match token.kind() {
            TokenKind::Keyword((Keyword::Function, _)) => {
                // FunctionDeclarations in IfStatement Statement Clauses
                // https://tc39.es/ecma262/#sec-functiondeclarations-in-ifstatement-statement-clauses
                if strict {
                    // This production only applies when parsing non-strict code.
                    return Err(Error::wrong_function_declaration_non_strict(position));
                }
                // Source text matched by this production is processed as if each matching
                // occurrence of FunctionDeclaration[?Yield, ?Await, ~Default] was the sole
                // StatementListItem of a BlockStatement occupying that position in the source text.
                Block::from(vec![StatementListItem::Declaration(Declaration::Function(
                    FunctionDeclaration::new(self.allow_yield, self.allow_await, false)
                        .parse(cursor, interner)?,
                ))])
                .into()
            }
            _ => Statement::new(self.allow_yield, self.allow_await, self.allow_return)
                .parse(cursor, interner)?,
        };

        // Early Error: It is a Syntax Error if IsLabelledFunction(the first Statement) is true.
        if then_node.is_labelled_function() {
            return Err(Error::wrong_labelled_function_declaration(position));
        }

        let else_stmt = if let Some(token) = cursor.peek(0, interner)? {
            match token.kind() {
                TokenKind::Keyword((Keyword::Else, true)) => {
                    return Err(Error::general(
                        "Keyword must not contain escaped characters",
                        token.span().start(),
                    ));
                }
                TokenKind::Keyword((Keyword::Else, false)) => {
                    cursor.advance(interner);

                    let strict = cursor.strict_mode();
                    let token = cursor.peek(0, interner).or_abrupt()?;
                    let position = token.span().start();
                    let stmt = match token.kind() {
                        TokenKind::Keyword((Keyword::Function, _)) => {
                            // FunctionDeclarations in IfStatement Statement Clauses
                            // https://tc39.es/ecma262/#sec-functiondeclarations-in-ifstatement-statement-clauses
                            if strict {
                                return Err(Error::wrong_function_declaration_non_strict(position));
                            }

                            // Source text matched by this production is processed as if each matching
                            // occurrence of FunctionDeclaration[?Yield, ?Await, ~Default] was the sole
                            // StatementListItem of a BlockStatement occupying that position in the source text.
                            Block::from(vec![StatementListItem::Declaration(
                                Declaration::Function(
                                    FunctionDeclaration::new(
                                        self.allow_yield,
                                        self.allow_await,
                                        false,
                                    )
                                    .parse(cursor, interner)?,
                                ),
                            )])
                            .into()
                        }
                        _ => Statement::new(self.allow_yield, self.allow_await, self.allow_return)
                            .parse(cursor, interner)?,
                    };

                    // Early Error: It is a Syntax Error if IsLabelledFunction(the second Statement) is true.
                    if stmt.is_labelled_function() {
                        return Err(Error::wrong_labelled_function_declaration(position));
                    }

                    Some(stmt)
                }
                _ => None,
            }
        } else {
            None
        };

        Ok(If::new(condition, then_node, else_stmt))
    }
}
