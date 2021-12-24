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
        ast::{node, node::declaration::Declaration, Punctuator},
        lexer::{Error as LexError, InputElement, TokenKind},
        parser::{
            expression::Initializer,
            statement::{
                ArrayBindingPattern, BindingIdentifier, ObjectBindingPattern, StatementList,
            },
            AllowAwait, AllowYield, Cursor, ParseError, TokenParser,
        },
    },
    BoaProfiler, Interner,
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
    type Output = FormalParameterList;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("FormalParameters", "Parsing");
        cursor.set_goal(InputElement::RegExp);

        let mut params = Vec::new();
        let mut is_simple = true;
        let mut has_duplicates = false;

        let next_token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
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
                return Err(ParseError::lex(LexError::Syntax(
                    "Rest parameter may not have a default initializer".into(),
                    start_position,
                )));
            }

            if next_param.is_rest_param()
                || next_param.init().is_some()
                || !next_param.is_identifier()
            {
                is_simple = false;
            }
            for param_name in next_param.names() {
                if parameter_names.contains(param_name) {
                    has_duplicates = true;
                }
                parameter_names.insert(Box::from(param_name));
            }
            params.push(next_param);

            if cursor
                .peek(0, interner)?
                .ok_or(ParseError::AbruptEnd)?
                .kind()
                == &TokenKind::Punctuator(Punctuator::CloseParen)
            {
                break;
            }

            if rest_param {
                let next = cursor.next(interner)?.expect("peeked token disappeared");
                return Err(ParseError::unexpected(
                    next.to_string(interner),
                    next.span(),
                    "rest parameter must be the last formal parameter",
                ));
            }

            cursor.expect(Punctuator::Comma, "parameter list", interner)?;
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

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("BindingRestElement", "Parsing");
        cursor.expect(Punctuator::Spread, "rest parameter", interner)?;

        if let Some(t) = cursor.peek(0, interner)? {
            let declaration = match *t.kind() {
                TokenKind::Punctuator(Punctuator::OpenBlock) => {
                    let param = ObjectBindingPattern::new(true, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;

                    let init = cursor
                        .peek(0, interner)?
                        .cloned()
                        .filter(|t| {
                            // Check that this is an initializer before attempting parse.
                            *t.kind() == TokenKind::Punctuator(Punctuator::Assign)
                        })
                        .map(|_| {
                            Initializer::new(true, self.allow_yield, self.allow_await)
                                .parse(cursor, interner)
                        })
                        .transpose()?;
                    Declaration::new_with_object_pattern(param, init)
                }

                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    Declaration::new_with_array_pattern(
                        ArrayBindingPattern::new(true, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?,
                        None,
                    )
                }

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
                            Initializer::new(true, self.allow_yield, self.allow_await)
                                .parse(cursor, interner)
                        })
                        .transpose()?;

                    Declaration::new_with_identifier(params, init)
                }
            };
            Ok(Self::Output::new(declaration, true))
        } else {
            Ok(Self::Output::new(
                Declaration::new_with_identifier("", None),
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
pub(in crate::syntax::parser) struct FormalParameter {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl FormalParameter {
    /// Creates a new `FormalParameter` parser.
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

impl<R> TokenParser<R> for FormalParameter
where
    R: Read,
{
    type Output = node::FormalParameter;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("FormalParameter", "Parsing");

        if let Some(t) = cursor.peek(0, interner)? {
            let declaration = match *t.kind() {
                TokenKind::Punctuator(Punctuator::OpenBlock) => {
                    let param = ObjectBindingPattern::new(true, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;

                    let init = cursor
                        .peek(0, interner)?
                        .cloned()
                        .filter(|t| {
                            // Check that this is an initializer before attempting parse.
                            *t.kind() == TokenKind::Punctuator(Punctuator::Assign)
                        })
                        .map(|_| {
                            Initializer::new(true, self.allow_yield, self.allow_await)
                                .parse(cursor, interner)
                        })
                        .transpose()?;

                    Declaration::new_with_object_pattern(param, init)
                }

                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    Declaration::new_with_array_pattern(
                        ArrayBindingPattern::new(true, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?,
                        None,
                    )
                }

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
                            Initializer::new(true, self.allow_yield, self.allow_await)
                                .parse(cursor, interner)
                        })
                        .transpose()?;

                    Declaration::new_with_identifier(params, init)
                }
            };
            Ok(Self::Output::new(declaration, false))
        } else {
            Ok(Self::Output::new(
                Declaration::new_with_identifier("", None),
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

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("FunctionStatementList", "Parsing");

        let global_strict_mode = cursor.strict_mode();
        let mut strict = false;

        if let Some(tk) = cursor.peek(0, interner)? {
            match tk.kind() {
                TokenKind::Punctuator(Punctuator::CloseBlock) => {
                    return Ok(Vec::new().into());
                }
                TokenKind::StringLiteral(string)
                    if interner.resolve(*string).expect("string disappeared") == "use strict" =>
                {
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
        .parse(cursor, interner);

        // Reset strict mode back to the global scope.
        cursor.set_strict_mode(global_strict_mode);

        let mut statement_list = statement_list?;
        statement_list.set_strict(strict);
        Ok(statement_list)
    }
}
