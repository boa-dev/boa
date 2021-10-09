//! Function definition parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function
//! [spec]: https://tc39.es/ecma262/#sec-function-definitions

#[cfg(test)]
mod tests;

use crate::{
    syntax::{
        ast::{node, Punctuator},
        lexer::{Error as LexError, InputElement, TokenKind},
        parser::{
            expression::Initializer,
            statement::{BindingIdentifier, StatementList},
            Cursor, ParseError, TokenParser,
        },
    },
    BoaProfiler,
};
use rustc_hash::FxHashSet;
use std::io::Read;

/// Intermediate type for a list of FormalParameters with some meta information.
pub(in crate::syntax::parser) struct FormalParameterList {
    pub(in crate::syntax::parser) parameters: Box<[node::FormalParameter]>,
    pub(in crate::syntax::parser) is_simple: bool,
    pub(in crate::syntax::parser) has_duplicates: bool,
}

/// Formal parameters parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/Parameter
/// [spec]: https://tc39.es/ecma262/#prod-FormalParameters
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct FormalParameters<const YIELD: bool, const AWAIT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for FormalParameters<YIELD, AWAIT>
where
    R: Read,
{
    type Output = FormalParameterList;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("FormalParameters", "Parsing");
        cursor.set_goal(InputElement::RegExp);

        let mut params = Vec::new();
        let mut is_simple = true;
        let mut has_duplicates = false;

        let next_token = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;
        if next_token.kind() == &TokenKind::Punctuator(Punctuator::CloseParen) {
            return Ok(FormalParameterList {
                parameters: params.into_boxed_slice(),
                is_simple,
                has_duplicates,
            });
        }
        let start_position = next_token.span().start();

        let mut parameter_names = FxHashSet::default();

        loop {
            let mut rest_param = false;

            let next_param = match cursor.peek(0)? {
                Some(tok) if tok.kind() == &TokenKind::Punctuator(Punctuator::Spread) => {
                    rest_param = true;
                    BindingRestElement::<YIELD, AWAIT>.parse(cursor)?
                }
                _ => FormalParameter::<YIELD, AWAIT>.parse(cursor)?,
            };

            if next_param.is_rest_param() || next_param.init().is_some() {
                is_simple = false;
            }
            if parameter_names.contains(next_param.name()) {
                has_duplicates = true;
            }
            parameter_names.insert(Box::from(next_param.name()));

            params.push(next_param);

            if cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind()
                == &TokenKind::Punctuator(Punctuator::CloseParen)
            {
                break;
            }

            if rest_param {
                return Err(ParseError::unexpected(
                    cursor.next()?.expect("peeked token disappeared"),
                    "rest parameter must be the last formal parameter",
                ));
            }

            cursor.expect(Punctuator::Comma, "parameter list")?;
        }

        // Early Error: It is a Syntax Error if IsSimpleParameterList of FormalParameterList is false
        // and BoundNames of FormalParameterList contains any duplicate elements.
        if !is_simple && has_duplicates {
            return Err(ParseError::lex(LexError::Syntax(
                "Duplicate parameter name not allowed in this context".into(),
                start_position,
            )));
        }

        Ok(FormalParameterList {
            parameters: params.into_boxed_slice(),
            is_simple,
            has_duplicates,
        })
    }
}

/// Rest parameter parsing.
///
/// It is equivalent to a `FunctionRestParameter`.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec_bind]
/// - [ECMAScript specification][spec_func]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/rest_parameters
/// [spec_bind]: https://tc39.es/ecma262/#prod-BindingRestElement
/// [spec_func]: https://tc39.es/ecma262/#prod-FunctionRestParameter
#[derive(Debug, Clone, Copy)]
struct BindingRestElement<const YIELD: bool, const AWAIT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for BindingRestElement<YIELD, AWAIT>
where
    R: Read,
{
    type Output = node::FormalParameter;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("BindingRestElement", "Parsing");
        cursor.expect(Punctuator::Spread, "rest parameter")?;

        let param = BindingIdentifier::<YIELD, AWAIT>.parse(cursor)?;
        // TODO: BindingPattern

        Ok(Self::Output::new(param, None, true))
    }
}

/// Formal parameter parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/Parameter
/// [spec]: https://tc39.es/ecma262/#prod-FormalParameter
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct FormalParameter<const YIELD: bool, const AWAIT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for FormalParameter<YIELD, AWAIT>
where
    R: Read,
{
    type Output = node::FormalParameter;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("FormalParameter", "Parsing");

        // TODO: BindingPattern

        let param = BindingIdentifier::<YIELD, AWAIT>.parse(cursor)?;

        let init = if let Some(t) = cursor.peek(0)? {
            // Check that this is an initilizer before attempting parse.
            if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                Some(Initializer::<true, YIELD, AWAIT>.parse(cursor)?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self::Output::new(param, init, false))
    }
}

/// The possible TokenKind which indicate the end of a function statement.
const FUNCTION_BREAK_TOKENS: [TokenKind; 1] = [TokenKind::Punctuator(Punctuator::CloseBlock)];

/// A function statement list.
///
/// A `FunctionStatementList` is equivalent to a `FunctionBody`.
///
/// More information:
///  - [ECMAScript specification][spec_list]
/// - [ECMAScript specification][spec_body]
///
/// [spec_list]: https://tc39.es/ecma262/#prod-FunctionStatementList
/// [spec_body]: https://tc39.es/ecma262/#prod-FunctionBody
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct FunctionStatementList<const YIELD: bool, const AWAIT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for FunctionStatementList<YIELD, AWAIT>
where
    R: Read,
{
    type Output = node::StatementList;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("FunctionStatementList", "Parsing");

        let global_strict_mode = cursor.strict_mode();
        let mut strict = false;

        if let Some(tk) = cursor.peek(0)? {
            match tk.kind() {
                TokenKind::Punctuator(Punctuator::CloseBlock) => {
                    return Ok(Vec::new().into());
                }
                TokenKind::StringLiteral(string) if string.as_ref() == "use strict" => {
                    cursor.set_strict_mode(true);
                    strict = true;
                }
                _ => {}
            }
        }

        let statement_list =
            StatementList::<YIELD, AWAIT, true, true>::new(&FUNCTION_BREAK_TOKENS).parse(cursor);

        // Reset strict mode back to the global scope.
        cursor.set_strict_mode(global_strict_mode);

        let mut statement_list = statement_list?;
        statement_list.set_strict(strict);
        Ok(statement_list)
    }
}
