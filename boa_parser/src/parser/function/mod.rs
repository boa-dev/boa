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
    lexer::{Error as LexError, InputElement, TokenKind},
    parser::{
        expression::{BindingIdentifier, Initializer},
        statement::{ArrayBindingPattern, ObjectBindingPattern, StatementList},
        AllowAwait, AllowYield, Cursor, OrAbrupt, ParseResult, TokenParser,
    },
    Error,
};
use ast::{
    operations::{check_labels, contains_invalid_object_literal},
    Position,
};
use boa_ast::{
    self as ast,
    declaration::Variable,
    function::{FormalParameterList, FormalParameterListFlags},
    Punctuator,
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::io::Read;

/// Formal parameters parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/Parameter
/// [spec]: https://tc39.es/ecma262/#prod-FormalParameters
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct FormalParameters {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl FormalParameters {
    /// Creates a new `FormalParameters` parser.
    pub(in crate::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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
    type Output = FormalParameterList;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("FormalParameters", "Parsing");

        cursor.set_goal(InputElement::RegExp);

        let Some(start_position) = cursor
            .peek(0, interner)?
            .filter(|&tok| tok.kind() != &TokenKind::Punctuator(Punctuator::CloseParen))
            .map(|tok| tok.span().start()) else {
            return Ok( FormalParameterList::default());
        };

        let mut params = Vec::new();

        loop {
            let mut rest_param = false;

            let next_param = match cursor.peek(0, interner)? {
                Some(tok) if tok.kind() == &TokenKind::Punctuator(Punctuator::Spread) => {
                    rest_param = true;
                    FunctionRestParameter::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?
                }
                _ => FormalParameter::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?,
            };

            if next_param.is_rest_param() && next_param.init().is_some() {
                return Err(Error::lex(LexError::Syntax(
                    "Rest parameter may not have a default initializer".into(),
                    start_position,
                )));
            }

            params.push(next_param);

            if cursor.peek(0, interner)?.map_or(true, |tok| {
                tok.kind() == &TokenKind::Punctuator(Punctuator::CloseParen)
            }) {
                break;
            }

            if rest_param {
                let next = cursor.next(interner)?.expect("peeked token disappeared");
                return Err(Error::unexpected(
                    next.to_string(interner),
                    next.span(),
                    "rest parameter must be the last formal parameter",
                ));
            }

            cursor.expect(Punctuator::Comma, "parameter list", interner)?;
            if cursor.peek(0, interner)?.map_or(true, |tok| {
                tok.kind() == &TokenKind::Punctuator(Punctuator::CloseParen)
            }) {
                break;
            }
        }

        let params = FormalParameterList::from_parameters(params);

        // Early Error: It is a Syntax Error if IsSimpleParameterList of FormalParameterList is false
        // and BoundNames of FormalParameterList contains any duplicate elements.
        if !params.flags().contains(FormalParameterListFlags::IS_SIMPLE)
            && params
                .flags()
                .contains(FormalParameterListFlags::HAS_DUPLICATES)
        {
            return Err(Error::lex(LexError::Syntax(
                "Duplicate parameter name not allowed in this context".into(),
                start_position,
            )));
        }
        Ok(params)
    }
}

/// `UniqueFormalParameters` parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-UniqueFormalParameters
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct UniqueFormalParameters {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl UniqueFormalParameters {
    /// Creates a new `UniqueFormalParameters` parser.
    pub(in crate::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for UniqueFormalParameters
where
    R: Read,
{
    type Output = FormalParameterList;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let params_start_position = cursor
            .expect(
                TokenKind::Punctuator(Punctuator::OpenParen),
                "unique formal parameters",
                interner,
            )?
            .span()
            .end();
        let params =
            FormalParameters::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;
        cursor.expect(
            TokenKind::Punctuator(Punctuator::CloseParen),
            "unique formal parameters",
            interner,
        )?;

        // Early Error: UniqueFormalParameters : FormalParameters
        if params.has_duplicates() {
            return Err(Error::lex(LexError::Syntax(
                "duplicate parameter name not allowed in unique formal parameters".into(),
                params_start_position,
            )));
        }
        Ok(params)
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
    type Output = ast::function::FormalParameter;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("BindingRestElement", "Parsing");
        cursor.expect(Punctuator::Spread, "rest parameter", interner)?;

        if let Some(t) = cursor.peek(0, interner)? {
            let declaration = match *t.kind() {
                TokenKind::Punctuator(Punctuator::OpenBlock) => {
                    let param = ObjectBindingPattern::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;

                    let init = cursor
                        .peek(0, interner)?
                        .cloned()
                        .filter(|t| {
                            // Check that this is an initializer before attempting parse.
                            *t.kind() == TokenKind::Punctuator(Punctuator::Assign)
                        })
                        .map(|_| {
                            Initializer::new(None, true, self.allow_yield, self.allow_await)
                                .parse(cursor, interner)
                        })
                        .transpose()?;
                    Variable::from_pattern(param.into(), init)
                }

                TokenKind::Punctuator(Punctuator::OpenBracket) => Variable::from_pattern(
                    ArrayBindingPattern::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?
                        .into(),
                    None,
                ),

                _ => {
                    let params = BindingIdentifier::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                    let init = cursor
                        .peek(0, interner)?
                        .cloned()
                        .filter(|t| {
                            // Check that this is an initializer before attempting parse.
                            *t.kind() == TokenKind::Punctuator(Punctuator::Assign)
                        })
                        .map(|_| {
                            Initializer::new(None, true, self.allow_yield, self.allow_await)
                                .parse(cursor, interner)
                        })
                        .transpose()?;

                    Variable::from_identifier(params, init)
                }
            };
            Ok(Self::Output::new(declaration, true))
        } else {
            Ok(Self::Output::new(
                Variable::from_identifier(Sym::EMPTY_STRING.into(), None),
                true,
            ))
        }
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
pub(in crate::parser) struct FormalParameter {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl FormalParameter {
    /// Creates a new `FormalParameter` parser.
    pub(in crate::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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
    type Output = ast::function::FormalParameter;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("FormalParameter", "Parsing");

        if let Some(t) = cursor.peek(0, interner)? {
            let declaration = match *t.kind() {
                TokenKind::Punctuator(Punctuator::OpenBlock) => {
                    let bindings = ObjectBindingPattern::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                    let init = if *cursor.peek(0, interner).or_abrupt()?.kind()
                        == TokenKind::Punctuator(Punctuator::Assign)
                    {
                        Some(
                            Initializer::new(None, true, self.allow_yield, self.allow_await)
                                .parse(cursor, interner)?,
                        )
                    } else {
                        None
                    };

                    Variable::from_pattern(bindings.into(), init)
                }
                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    let bindings = ArrayBindingPattern::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                    let init = if *cursor.peek(0, interner).or_abrupt()?.kind()
                        == TokenKind::Punctuator(Punctuator::Assign)
                    {
                        Some(
                            Initializer::new(None, true, self.allow_yield, self.allow_await)
                                .parse(cursor, interner)?,
                        )
                    } else {
                        None
                    };

                    Variable::from_pattern(bindings.into(), init)
                }
                _ => {
                    let ident = BindingIdentifier::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                    let init = if cursor.peek(0, interner)?.map_or(false, |tok| {
                        tok.kind() == &TokenKind::Punctuator(Punctuator::Assign)
                    }) {
                        Some(
                            Initializer::new(None, true, self.allow_yield, self.allow_await)
                                .parse(cursor, interner)?,
                        )
                    } else {
                        None
                    };

                    Variable::from_identifier(ident, init)
                }
            };
            Ok(Self::Output::new(declaration, false))
        } else {
            Ok(Self::Output::new(
                Variable::from_identifier(Sym::EMPTY_STRING.into(), None),
                false,
            ))
        }
    }
}

/// A `FunctionBody` is equivalent to a `FunctionStatementList`.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FunctionBody
pub(in crate::parser) type FunctionBody = FunctionStatementList;

/// The possible `TokenKind` which indicate the end of a function statement.
pub(in crate::parser) const FUNCTION_BREAK_TOKENS: [TokenKind; 1] =
    [TokenKind::Punctuator(Punctuator::CloseBlock)];

/// A function statement list
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FunctionStatementList
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct FunctionStatementList {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl FunctionStatementList {
    /// Creates a new `FunctionStatementList` parser.
    pub(in crate::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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
    type Output = ast::StatementList;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("FunctionStatementList", "Parsing");

        let statement_list = StatementList::new(
            self.allow_yield,
            self.allow_await,
            true,
            &FUNCTION_BREAK_TOKENS,
            true,
            false,
        )
        .parse(cursor, interner)?;

        if let Err(error) = check_labels(&statement_list) {
            return Err(Error::lex(LexError::Syntax(
                error.message(interner).into(),
                Position::new(1, 1),
            )));
        }

        if contains_invalid_object_literal(&statement_list) {
            return Err(Error::lex(LexError::Syntax(
                "invalid object literal in function statement list".into(),
                Position::new(1, 1),
            )));
        }

        Ok(statement_list)
    }
}
