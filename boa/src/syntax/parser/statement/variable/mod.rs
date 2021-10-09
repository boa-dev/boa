//! Variable statement parsing.

use crate::{
    syntax::{
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
            ParseError, TokenParser,
        },
    },
    BoaProfiler,
};
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
pub(in crate::syntax::parser::statement) struct VariableStatement<
    const YIELD: bool,
    const AWAIT: bool,
>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for VariableStatement<YIELD, AWAIT>
where
    R: Read,
{
    type Output = DeclarationList;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("VariableStatement", "Parsing");
        cursor.expect(Keyword::Var, "variable statement")?;

        let decl_list = VariableDeclarationList::<true, YIELD, AWAIT>.parse(cursor)?;

        cursor.expect_semicolon("variable statement")?;

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
pub(in crate::syntax::parser::statement) struct VariableDeclarationList<
    const IN: bool,
    const YIELD: bool,
    const AWAIT: bool,
>;

impl<R, const IN: bool, const YIELD: bool, const AWAIT: bool> TokenParser<R>
    for VariableDeclarationList<IN, YIELD, AWAIT>
where
    R: Read,
{
    type Output = DeclarationList;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let mut list = Vec::new();

        loop {
            list.push(VariableDeclaration::<IN, YIELD, AWAIT>.parse(cursor)?);

            match cursor.peek_semicolon()? {
                SemicolonResult::NotFound(tk)
                    if tk.kind() == &TokenKind::Punctuator(Punctuator::Comma) =>
                {
                    let _ = cursor.next();
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
struct VariableDeclaration<const IN: bool, const YIELD: bool, const AWAIT: bool>;

impl<R, const IN: bool, const YIELD: bool, const AWAIT: bool> TokenParser<R>
    for VariableDeclaration<IN, YIELD, AWAIT>
where
    R: Read,
{
    type Output = Declaration;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let peek_token = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

        match peek_token.kind() {
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                let bindings = ObjectBindingPattern::<IN, YIELD, AWAIT>.parse(cursor)?;

                let init = if let Some(t) = cursor.peek(0)? {
                    if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                        Some(Initializer::<IN, YIELD, AWAIT>.parse(cursor)?)
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Declaration::new_with_object_pattern(bindings, init))
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                let bindings = ArrayBindingPattern::<IN, YIELD, AWAIT>.parse(cursor)?;

                let init = if let Some(t) = cursor.peek(0)? {
                    if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                        Some(Initializer::<IN, YIELD, AWAIT>.parse(cursor)?)
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Declaration::new_with_array_pattern(bindings, init))
            }

            _ => {
                let ident = BindingIdentifier::<YIELD, AWAIT>.parse(cursor)?;

                let init = if let Some(t) = cursor.peek(0)? {
                    if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                        Some(Initializer::<true, YIELD, AWAIT>.parse(cursor)?)
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
