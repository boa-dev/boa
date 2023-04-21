//! Variable statement parsing.

use crate::{
    lexer::TokenKind,
    parser::{
        cursor::Cursor,
        expression::Initializer,
        statement::{ArrayBindingPattern, BindingIdentifier, ObjectBindingPattern},
        AllowAwait, AllowIn, AllowYield, OrAbrupt, ParseResult, TokenParser,
    },
};
use boa_ast::{
    declaration::{VarDeclaration, Variable},
    Keyword, Punctuator,
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::{convert::TryInto, io::Read};

/// Variable statement parsing.
///
/// A variable statement contains the `var` keyword.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/var
/// [spec]: https://tc39.es/ecma262/#prod-VariableStatement
#[derive(Debug, Clone, Copy)]
pub(in crate::parser::statement) struct VariableStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl VariableStatement {
    /// Creates a new `VariableStatement` parser.
    pub(in crate::parser::statement) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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
    type Output = VarDeclaration;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("VariableStatement", "Parsing");
        cursor.expect((Keyword::Var, false), "variable statement", interner)?;

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
pub(in crate::parser::statement) struct VariableDeclarationList {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl VariableDeclarationList {
    /// Creates a new `VariableDeclarationList` parser.
    pub(in crate::parser::statement) fn new<I, Y, A>(
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
    type Output = VarDeclaration;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let mut list = Vec::new();

        loop {
            list.push(
                VariableDeclaration::new(self.allow_in, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?,
            );

            if cursor.next_if(Punctuator::Comma, interner)?.is_none() {
                break;
            }
        }

        Ok(VarDeclaration(list.try_into().expect(
            "`VariableDeclaration` must parse at least one variable",
        )))
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
    type Output = Variable;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let peek_token = cursor.peek(0, interner).or_abrupt()?;

        match peek_token.kind() {
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                let bindings = ObjectBindingPattern::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;

                let init = if cursor
                    .peek(0, interner)?
                    .filter(|t| *t.kind() == TokenKind::Punctuator(Punctuator::Assign))
                    .is_some()
                {
                    Some(
                        Initializer::new(None, self.allow_in, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?,
                    )
                } else {
                    None
                };

                Ok(Variable::from_pattern(bindings.into(), init))
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                let bindings = ArrayBindingPattern::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;

                let init = if cursor
                    .peek(0, interner)?
                    .filter(|t| *t.kind() == TokenKind::Punctuator(Punctuator::Assign))
                    .is_some()
                {
                    Some(
                        Initializer::new(None, self.allow_in, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?,
                    )
                } else {
                    None
                };

                Ok(Variable::from_pattern(bindings.into(), init))
            }
            _ => {
                let ident = BindingIdentifier::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;

                let init = if cursor
                    .peek(0, interner)?
                    .filter(|t| *t.kind() == TokenKind::Punctuator(Punctuator::Assign))
                    .is_some()
                {
                    Some(
                        Initializer::new(
                            Some(ident),
                            self.allow_in,
                            self.allow_yield,
                            self.allow_await,
                        )
                        .parse(cursor, interner)?,
                    )
                } else {
                    None
                };
                Ok(Variable::from_identifier(ident, init))
            }
        }
    }
}
