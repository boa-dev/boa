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
        lexer::{InputElement, TokenKind},
        parser::{
            expression::Initializer,
            statement::{BindingIdentifier, StatementList},
            AllowAwait, AllowYield, Cursor, ParseError, TokenParser,
        },
    },
    BoaProfiler,
};
use std::{collections::HashSet, io::Read};

/// Formal parameters parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/Parameter
/// [spec]: https://tc39.es/ecma262/#prod-FormalParameters
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct FormalParameters {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl FormalParameters {
    /// Creates a new `FormalParameters` parser.
    pub(in crate::syntax::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for FormalParameters
where
    R: Read,
{
    type Output = Box<[node::FormalParameter]>;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("FormalParameters", "Parsing");
        cursor.set_goal(InputElement::RegExp);

        let mut params = Vec::new();
        let mut param_names = HashSet::new();

        if cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind()
            == &TokenKind::Punctuator(Punctuator::CloseParen)
        {
            return Ok(params.into_boxed_slice());
        }

        loop {
            let mut rest_param = false;

            let position = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.span().start();
            let next_param = match cursor.peek(0)? {
                Some(tok) if tok.kind() == &TokenKind::Punctuator(Punctuator::Spread) => {
                    rest_param = true;
                    FunctionRestParameter::new(self.allow_yield, self.allow_await).parse(cursor)?
                }
                _ => FormalParameter::new(self.allow_yield, self.allow_await).parse(cursor)?,
            };
            if param_names.contains(next_param.name()) {
                return Err(ParseError::general("duplicate parameter name", position));
            }

            param_names.insert(Box::from(next_param.name()));
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

        Ok(params.into_boxed_slice())
    }
}

/// Rest parameter parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/rest_parameters
/// [spec]: https://tc39.es/ecma262/#prod-FunctionRestParameter
type FunctionRestParameter = BindingRestElement;

/// Rest parameter parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/rest_parameters
/// [spec]: https://tc39.es/ecma262/#prod-BindingRestElement
#[derive(Debug, Clone, Copy)]
struct BindingRestElement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl BindingRestElement {
    /// Creates a new `BindingRestElement` parser.
    fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for BindingRestElement
where
    R: Read,
{
    type Output = node::FormalParameter;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("BindingRestElement", "Parsing");
        cursor.expect(Punctuator::Spread, "rest parameter")?;

        let param = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;
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
struct FormalParameter {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl FormalParameter {
    /// Creates a new `FormalParameter` parser.
    fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for FormalParameter
where
    R: Read,
{
    type Output = node::FormalParameter;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("FormalParameter", "Parsing");

        // TODO: BindingPattern

        let param = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;

        let init = if let Some(t) = cursor.peek(0)? {
            // Check that this is an initilizer before attempting parse.
            if *t.kind() == TokenKind::Punctuator(Punctuator::Assign) {
                Some(Initializer::new(true, self.allow_yield, self.allow_await).parse(cursor)?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self::Output::new(param, init, false))
    }
}

/// A `FunctionBody` is equivalent to a `FunctionStatementList`.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FunctionBody
pub(in crate::syntax::parser) type FunctionBody = FunctionStatementList;

/// The possible TokenKind which indicate the end of a function statement.
const FUNCTION_BREAK_TOKENS: [TokenKind; 1] = [TokenKind::Punctuator(Punctuator::CloseBlock)];

/// A function statement list
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FunctionStatementList
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct FunctionStatementList {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl FunctionStatementList {
    /// Creates a new `FunctionStatementList` parser.
    pub(in crate::syntax::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for FunctionStatementList
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

        let statement_list = StatementList::new(
            self.allow_yield,
            self.allow_await,
            true,
            true,
            &FUNCTION_BREAK_TOKENS,
        )
        .parse(cursor);

        // Reset strict mode back to the global scope.
        cursor.set_strict_mode(global_strict_mode);

        let mut statement_list = statement_list?;
        statement_list.set_strict(strict);
        Ok(statement_list)
    }
}
