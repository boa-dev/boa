//! For statement parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for
//! [spec]: https://tc39.es/ecma262/#sec-for-statement

use crate::syntax::{
    ast::{
        node::{
            iteration::IterableLoopInitializer,
            operator::assign::{
                array_decl_to_declaration_pattern, object_decl_to_declaration_pattern,
            },
            ForInLoop, ForLoop, ForOfLoop, Node,
        },
        Const, Keyword, Position, Punctuator,
    },
    lexer::{Error as LexError, TokenKind},
    parser::{
        expression::Expression,
        statement::declaration::Declaration,
        statement::{variable::VariableDeclarationList, Statement},
        AllowAwait, AllowReturn, AllowYield, Cursor, ParseError, TokenParser,
    },
};
use boa_interner::Interner;
use boa_profiler::Profiler;
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

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("ForStatement", "Parsing");
        cursor.expect(Keyword::For, "for statement", interner)?;
        let init_position = cursor
            .expect(Punctuator::OpenParen, "for statement", interner)?
            .span()
            .end();

        let init = match cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .kind()
        {
            TokenKind::Keyword(Keyword::Var) => {
                let _next = cursor.next(interner)?;
                Some(
                    VariableDeclarationList::new(false, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)
                        .map(Node::from)?,
                )
            }
            TokenKind::Keyword(Keyword::Let | Keyword::Const) => Some(
                Declaration::new(self.allow_yield, self.allow_await, false)
                    .parse(cursor, interner)?,
            ),
            TokenKind::Punctuator(Punctuator::Semicolon) => None,
            _ => Some(
                Expression::new(None, false, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?,
            ),
        };

        match (init.as_ref(), cursor.peek(0, interner)?) {
            (Some(init), Some(tok)) if tok.kind() == &TokenKind::Keyword(Keyword::In) => {
                let init = node_to_iterable_loop_initializer(init, init_position)?;

                let _next = cursor.next(interner)?;
                let expr = Expression::new(None, true, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;

                let position = cursor
                    .expect(Punctuator::CloseParen, "for in statement", interner)?
                    .span()
                    .end();

                let body = Statement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor, interner)?;

                // Early Error: It is a Syntax Error if IsLabelledFunction(the first Statement) is true.
                if let Node::FunctionDecl(_) = body {
                    return Err(ParseError::wrong_function_declaration_non_strict(position));
                }

                return Ok(ForInLoop::new(init, expr, body).into());
            }
            (Some(init), Some(tok)) if tok.kind() == &TokenKind::Keyword(Keyword::Of) => {
                let init = node_to_iterable_loop_initializer(init, init_position)?;

                let _next = cursor.next(interner)?;
                let iterable = Expression::new(None, true, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;

                let position = cursor
                    .expect(Punctuator::CloseParen, "for of statement", interner)?
                    .span()
                    .end();

                let body = Statement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor, interner)?;

                // Early Error: It is a Syntax Error if IsLabelledFunction(the first Statement) is true.
                if let Node::FunctionDecl(_) = body {
                    return Err(ParseError::wrong_function_declaration_non_strict(position));
                }

                return Ok(ForOfLoop::new(init, iterable, body).into());
            }
            (Some(Node::ConstDeclList(list)), _) => {
                // Reject const declarations without initializers inside for loops
                for decl in list.as_ref() {
                    if decl.init().is_none() {
                        return Err(ParseError::general(
                            "Expected initializer for const declaration",
                            // TODO: get exact position of uninitialized const decl
                            init_position,
                        ));
                    }
                }
            }
            _ => {}
        }

        cursor.expect(Punctuator::Semicolon, "for statement", interner)?;

        let cond = if cursor.next_if(Punctuator::Semicolon, interner)?.is_some() {
            Const::from(true).into()
        } else {
            let step = Expression::new(None, true, self.allow_yield, self.allow_await)
                .parse(cursor, interner)?;
            cursor.expect(Punctuator::Semicolon, "for statement", interner)?;
            step
        };

        let step = if cursor.next_if(Punctuator::CloseParen, interner)?.is_some() {
            None
        } else {
            let step = Expression::new(None, true, self.allow_yield, self.allow_await)
                .parse(cursor, interner)?;
            cursor.expect(
                TokenKind::Punctuator(Punctuator::CloseParen),
                "for statement",
                interner,
            )?;
            Some(step)
        };

        let position = cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .span()
            .start();

        let body = Statement::new(self.allow_yield, self.allow_await, self.allow_return)
            .parse(cursor, interner)?;

        // Early Error: It is a Syntax Error if IsLabelledFunction(the first Statement) is true.
        if let Node::FunctionDecl(_) = body {
            return Err(ParseError::wrong_function_declaration_non_strict(position));
        }

        // TODO: do not encapsulate the `for` in a block just to have an inner scope.
        Ok(ForLoop::new(init, cond, step, body).into())
    }
}

#[inline]
fn node_to_iterable_loop_initializer(
    node: &Node,
    position: Position,
) -> Result<IterableLoopInitializer, ParseError> {
    match node {
        Node::Identifier(name) => Ok(IterableLoopInitializer::Identifier(*name)),
        Node::VarDeclList(ref list) => match list.as_ref() {
            [var] => {
                if var.init().is_some() {
                    return Err(ParseError::lex(LexError::Syntax(
                        "a declaration in the head of a for-of loop can't have an initializer"
                            .into(),
                        position,
                    )));
                }
                Ok(IterableLoopInitializer::Var(var.clone()))
            }
            _ => Err(ParseError::lex(LexError::Syntax(
                "only one variable can be declared in the head of a for-of loop".into(),
                position,
            ))),
        },
        Node::LetDeclList(ref list) => match list.as_ref() {
            [var] => {
                if var.init().is_some() {
                    return Err(ParseError::lex(LexError::Syntax(
                        "a declaration in the head of a for-of loop can't have an initializer"
                            .into(),
                        position,
                    )));
                }
                Ok(IterableLoopInitializer::Let(var.clone()))
            }
            _ => Err(ParseError::lex(LexError::Syntax(
                "only one variable can be declared in the head of a for-of loop".into(),
                position,
            ))),
        },
        Node::ConstDeclList(ref list) => match list.as_ref() {
            [var] => {
                if var.init().is_some() {
                    return Err(ParseError::lex(LexError::Syntax(
                        "a declaration in the head of a for-of loop can't have an initializer"
                            .into(),
                        position,
                    )));
                }
                Ok(IterableLoopInitializer::Const(var.clone()))
            }
            _ => Err(ParseError::lex(LexError::Syntax(
                "only one variable can be declared in the head of a for-of loop".into(),
                position,
            ))),
        },
        Node::Assign(_) => Err(ParseError::lex(LexError::Syntax(
            "a declaration in the head of a for-of loop can't have an initializer".into(),
            position,
        ))),
        Node::Object(object) => {
            if let Some(pattern) = object_decl_to_declaration_pattern(object) {
                Ok(IterableLoopInitializer::DeclarationPattern(pattern))
            } else {
                Err(ParseError::lex(LexError::Syntax(
                    "invalid left-hand side declaration pattern in assignment head of a for-of loop".into(),
                    position,
                )))
            }
        }
        Node::ArrayDecl(array) => {
            if let Some(pattern) = array_decl_to_declaration_pattern(array) {
                Ok(IterableLoopInitializer::DeclarationPattern(pattern))
            } else {
                Err(ParseError::lex(LexError::Syntax(
                    "invalid left-hand side declaration pattern in assignment head of a for-of loop".into(),
                    position,
                )))
            }
        }
        _ => Err(ParseError::lex(LexError::Syntax(
            "unknown left hand side in head of for-of loop".into(),
            position,
        ))),
    }
}
