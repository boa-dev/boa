//! Variable statement parsing.

use crate::syntax::{
    ast::{
        node::{Declaration, DeclarationList},
        Keyword, Punctuator,
    },
    lexer::TokenKind,
    parser::statement::{ArrayBindingPattern, ObjectBindingPattern},
    parser::{
        cursor::{Cursor, SemicolonResult},
        expression::Initializer,
        statement::BindingIdentifier,
        AllowAwait, AllowIn, AllowYield, ParseError, TokenParser,
    },
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// Variable statement parsing.
///
/// A varible statement contains the `var` keyword.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/var
/// [spec]: https://tc39.es/ecma262/#prod-VariableStatement
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser::statement) struct VariableStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl VariableStatement {
    /// Creates a new `VariableStatement` parser.
    pub(in crate::syntax::parser::statement) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for VariableStatement
where
    R: Read,
{
    type Output = DeclarationList;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("VariableStatement", "Parsing");
        cursor.expect(Keyword::Var, "variable statement", interner)?;

        let decl_list = VariableDeclarationList::new(true, self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;

        cursor.expect_semicolon("variable statement", interner)?;

        Ok(decl_list)
    }
}

/// Variable declaration list parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/var
/// [spec]: https://tc39.es/ecma262/#prod-VariableDeclarationList
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser::statement) struct VariableDeclarationList {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl VariableDeclarationList {
    /// Creates a new `VariableDeclarationList` parser.
    pub(in crate::syntax::parser::statement) fn new<I, Y, A>(
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
    ) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for VariableDeclarationList
where
    R: Read,
{
    type Output = DeclarationList;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let mut list = Vec::new();

        loop {
            list.push(
                VariableDeclaration::new(self.allow_in, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?,
            );

            match cursor.peek_semicolon(interner)? {
                SemicolonResult::NotFound(tk)
                    if tk.kind() == &TokenKind::Punctuator(Punctuator::Comma) =>
                {
                    let _next = cursor.next(interner).expect("token disappeared");
                }
                _ => break,
            }
        }

        Ok(DeclarationList::Var(list.into()))
    }
}

/// Reads an individual variable declaration.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-VariableDeclaration
#[derive(Debug, Clone, Copy)]
struct VariableDeclaration {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl VariableDeclaration {
    /// Creates a new `VariableDeclaration` parser.
    fn new<I, Y, A>(allow_in: I, allow_yield: Y, allow_await: A) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for VariableDeclaration
where
    R: Read,
{
    type Output = Declaration;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let peek_token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;

        match peek_token.kind() {
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                let bindings =
                    ObjectBindingPattern::new(self.allow_in, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;

                let init = if let Some(t) = cursor.peek(0, interner)? {
                    if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                        Some(
                            Initializer::new(
                                None,
                                self.allow_in,
                                self.allow_yield,
                                self.allow_await,
                            )
                            .parse(cursor, interner)?,
                        )
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Declaration::new_with_object_pattern(bindings, init))
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                let bindings =
                    ArrayBindingPattern::new(self.allow_in, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;

                let init = if let Some(t) = cursor.peek(0, interner)? {
                    if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                        Some(
                            Initializer::new(
                                None,
                                self.allow_in,
                                self.allow_yield,
                                self.allow_await,
                            )
                            .parse(cursor, interner)?,
                        )
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Declaration::new_with_array_pattern(bindings, init))
            }

            _ => {
                let ident = BindingIdentifier::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;

                let init = if let Some(t) = cursor.peek(0, interner)? {
                    if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                        Some(
                            Initializer::new(Some(ident), true, self.allow_yield, self.allow_await)
                                .parse(cursor, interner)?,
                        )
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Declaration::new_with_identifier(ident, init))
            }
        }
    }
}
