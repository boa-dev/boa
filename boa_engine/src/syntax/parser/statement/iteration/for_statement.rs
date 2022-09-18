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
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use rustc_hash::FxHashSet;
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
        cursor.expect((Keyword::For, false), "for statement", interner)?;

        let mut r#await = false;

        let next = cursor.next(interner)?.ok_or(ParseError::AbruptEnd)?;
        let init_position = match next.kind() {
            TokenKind::Punctuator(Punctuator::OpenParen) => next.span().end(),
            TokenKind::Keyword((Keyword::Await, _)) if !self.allow_await.0 => {
                return Err(ParseError::unexpected(
                    next.to_string(interner),
                    next.span(),
                    "for await...of is only valid in async functions or async generators",
                ));
            }
            TokenKind::Keyword((Keyword::Await, _)) => {
                r#await = true;
                cursor
                    .expect(Punctuator::OpenParen, "for await...of", interner)?
                    .span()
                    .end()
            }
            _ => {
                return Err(ParseError::unexpected(
                    next.to_string(interner),
                    next.span(),
                    "for statement",
                ));
            }
        };

        let init = match cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .kind()
        {
            TokenKind::Keyword((Keyword::Var, _)) => {
                let _next = cursor.next(interner)?;
                Some(
                    VariableDeclarationList::new(false, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)
                        .map(Node::from)?,
                )
            }
            TokenKind::Keyword((Keyword::Let | Keyword::Const, _)) => Some(
                Declaration::new(self.allow_yield, self.allow_await, false)
                    .parse(cursor, interner)?,
            ),
            TokenKind::Keyword((Keyword::Async, false)) => {
                match cursor
                    .peek(1, interner)?
                    .ok_or(ParseError::AbruptEnd)?
                    .kind()
                {
                    TokenKind::Keyword((Keyword::Of, _)) => {
                        return Err(ParseError::lex(LexError::Syntax(
                            "invalid left-hand side expression 'async' of a for-of loop".into(),
                            init_position,
                        )));
                    }
                    _ => Some(
                        Expression::new(None, false, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?,
                    ),
                }
            }
            TokenKind::Punctuator(Punctuator::Semicolon) => None,
            _ => Some(
                Expression::new(None, false, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?,
            ),
        };

        let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
        match (init.as_ref(), token.kind()) {
            (Some(_), TokenKind::Keyword((Keyword::In | Keyword::Of, true))) => {
                return Err(ParseError::general(
                    "Keyword must not contain escaped characters",
                    token.span().start(),
                ));
            }
            (Some(init), TokenKind::Keyword((Keyword::In, false))) => {
                let init_position = token.span().start();
                let init =
                    node_to_iterable_loop_initializer(init, init_position, cursor.strict_mode())?;

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

                // It is a Syntax Error if the BoundNames of ForDeclaration contains "let".
                // It is a Syntax Error if any element of the BoundNames of ForDeclaration also occurs in the VarDeclaredNames of Statement.
                // It is a Syntax Error if the BoundNames of ForDeclaration contains any duplicate entries.
                let mut vars = FxHashSet::default();
                body.var_declared_names(&mut vars);
                let mut bound_names = FxHashSet::default();
                for name in init.bound_names() {
                    if name == Sym::LET {
                        return Err(ParseError::general(
                            "Cannot use 'let' as a lexically bound name",
                            init_position,
                        ));
                    }
                    if vars.contains(&name) {
                        return Err(ParseError::general(
                            "For loop initializer declared in loop body",
                            init_position,
                        ));
                    }
                    if !bound_names.insert(name) {
                        return Err(ParseError::general(
                            "For loop initializer cannot contain duplicate identifiers",
                            init_position,
                        ));
                    }
                }

                return Ok(ForInLoop::new(init, expr, body).into());
            }
            (Some(init), TokenKind::Keyword((Keyword::Of, false))) => {
                let init =
                    node_to_iterable_loop_initializer(init, init_position, cursor.strict_mode())?;

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

                // It is a Syntax Error if the BoundNames of ForDeclaration contains "let".
                // It is a Syntax Error if any element of the BoundNames of ForDeclaration also occurs in the VarDeclaredNames of Statement.
                // It is a Syntax Error if the BoundNames of ForDeclaration contains any duplicate entries.
                let mut vars = FxHashSet::default();
                body.var_declared_names(&mut vars);
                let mut bound_names = FxHashSet::default();
                for name in init.bound_names() {
                    if name == Sym::LET {
                        return Err(ParseError::general(
                            "Cannot use 'let' as a lexically bound name",
                            init_position,
                        ));
                    }
                    if vars.contains(&name) {
                        return Err(ParseError::general(
                            "For loop initializer declared in loop body",
                            init_position,
                        ));
                    }
                    if !bound_names.insert(name) {
                        return Err(ParseError::general(
                            "For loop initializer cannot contain duplicate identifiers",
                            init_position,
                        ));
                    }
                }

                return Ok(ForOfLoop::new(init, iterable, body, r#await).into());
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
    strict: bool,
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
            if let Some(pattern) = object_decl_to_declaration_pattern(object, strict) {
                Ok(IterableLoopInitializer::DeclarationPattern(pattern))
            } else {
                Err(ParseError::lex(LexError::Syntax(
                    "invalid left-hand side declaration pattern in assignment head of a for-of loop".into(),
                    position,
                )))
            }
        }
        Node::ArrayDecl(array) => {
            if let Some(pattern) = array_decl_to_declaration_pattern(array, strict) {
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
