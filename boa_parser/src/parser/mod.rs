//! Boa parser implementation.

mod cursor;
mod expression;
mod statement;

pub(crate) mod function;

#[cfg(test)]
mod tests;

use crate::{
    error::ParseResult,
    lexer::TokenKind,
    parser::{
        cursor::Cursor,
        function::{FormalParameters, FunctionStatementList},
    },
    Error,
};
use boa_ast::{
    expression::Identifier,
    function::FormalParameterList,
    operations::{
        contains, top_level_lexically_declared_names, top_level_var_declared_names, ContainsSymbol,
    },
    Position, StatementList,
};
use boa_interner::Interner;
use boa_macros::utf16;
use rustc_hash::FxHashSet;
use std::io::Read;

/// Trait implemented by parsers.
///
/// This makes it possible to abstract over the underlying implementation of a parser.
trait TokenParser<R>: Sized
where
    R: Read,
{
    /// Output type for the parser.
    type Output; // = Node; waiting for https://github.com/rust-lang/rust/issues/29661

    /// Parses the token stream using the current parser.
    ///
    /// This method needs to be provided by the implementor type.
    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output>;
}

/// Boolean representing if the parser should allow a `yield` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowYield(bool);

impl From<bool> for AllowYield {
    #[inline]
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if the parser should allow a `await` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowAwait(bool);

impl From<bool> for AllowAwait {
    #[inline]
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if the parser should allow a `in` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowIn(bool);

impl From<bool> for AllowIn {
    #[inline]
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if the parser should allow a `return` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowReturn(bool);

impl From<bool> for AllowReturn {
    #[inline]
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if the parser should allow a `default` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowDefault(bool);

impl From<bool> for AllowDefault {
    #[inline]
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Parser for the ECMAScript language.
///
/// This parser implementation tries to be conformant to the most recent
/// [ECMAScript language specification], and it also implements some legacy features like
/// [labelled functions][label] or [duplicated block-level function definitions][block].
///
/// [spec]: https://tc39.es/ecma262/#sec-ecmascript-language-source-code
/// [label]: https://tc39.es/ecma262/#sec-labelled-function-declarations
/// [block]: https://tc39.es/ecma262/#sec-block-duplicates-allowed-static-semantics
#[derive(Debug)]
pub struct Parser<R> {
    /// Cursor of the parser, pointing to the lexer and used to get tokens for the parser.
    cursor: Cursor<R>,
}

impl<R> Parser<R> {
    /// Create a new `Parser` with a reader as the input to parse.
    #[inline]
    pub fn new(reader: R) -> Self
    where
        R: Read,
    {
        Self {
            cursor: Cursor::new(reader),
        }
    }

    /// Set the parser strict mode to true.
    #[inline]
    pub fn set_strict(&mut self)
    where
        R: Read,
    {
        self.cursor.set_strict_mode(true);
    }

    /// Set the parser strict mode to true.
    #[inline]
    pub fn set_json_parse(&mut self)
    where
        R: Read,
    {
        self.cursor.set_json_parse(true);
    }

    /// Parse the full input as a [ECMAScript Script][spec] into the boa AST representation.
    /// The resulting `StatementList` can be compiled into boa bytecode and executed in the boa vm.
    ///
    /// # Errors
    ///
    /// Will return `Err` on any parsing error, including invalid reads of the bytes being parsed.
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-Script
    #[inline]
    pub fn parse_all(&mut self, interner: &mut Interner) -> ParseResult<StatementList>
    where
        R: Read,
    {
        Script::new(false).parse(&mut self.cursor, interner)
    }

    /// [`19.2.1.1 PerformEval ( x, strictCaller, direct )`][spec]
    ///
    /// Parses the source text input of an `eval` call.
    ///
    /// # Errors
    ///
    /// Will return `Err` on any parsing error, including invalid reads of the bytes being parsed.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-performeval
    #[inline]
    pub fn parse_eval(
        &mut self,
        direct: bool,
        interner: &mut Interner,
    ) -> ParseResult<StatementList>
    where
        R: Read,
    {
        Script::new(direct).parse(&mut self.cursor, interner)
    }

    /// Parses the full input as an [ECMAScript `FunctionBody`][spec] into the boa AST representation.
    ///
    /// # Errors
    ///
    /// Will return `Err` on any parsing error, including invalid reads of the bytes being parsed.
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-FunctionBody
    #[inline]
    pub fn parse_function_body(
        &mut self,
        interner: &mut Interner,
        allow_yield: bool,
        allow_await: bool,
    ) -> ParseResult<StatementList>
    where
        R: Read,
    {
        FunctionStatementList::new(allow_yield, allow_await).parse(&mut self.cursor, interner)
    }

    /// Parses the full input as an [ECMAScript `FormalParameterList`][spec] into the boa AST representation.
    ///
    /// # Errors
    ///
    /// Will return `Err` on any parsing error, including invalid reads of the bytes being parsed.
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-FormalParameterList
    #[inline]
    pub fn parse_formal_parameters(
        &mut self,
        interner: &mut Interner,
        allow_yield: bool,
        allow_await: bool,
    ) -> ParseResult<FormalParameterList>
    where
        R: Read,
    {
        FormalParameters::new(allow_yield, allow_await).parse(&mut self.cursor, interner)
    }
}

/// Parses a full script.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-Script
#[derive(Debug, Clone, Copy)]
pub struct Script {
    direct_eval: bool,
}

impl Script {
    /// Create a new `Script` parser.
    #[inline]
    const fn new(direct_eval: bool) -> Self {
        Self { direct_eval }
    }
}

impl<R> TokenParser<R> for Script
where
    R: Read,
{
    type Output = StatementList;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let mut strict = cursor.strict_mode();
        match cursor.peek(0, interner)? {
            Some(tok) => {
                match tok.kind() {
                    // Set the strict mode
                    TokenKind::StringLiteral(string)
                        if interner.resolve_expect(*string).join(
                            |s| s == "use strict",
                            |g| g == utf16!("use strict"),
                            true,
                        ) =>
                    {
                        cursor.set_strict_mode(true);
                        strict = true;
                    }
                    _ => {}
                }
                let mut statement_list =
                    ScriptBody::new(self.direct_eval).parse(cursor, interner)?;
                statement_list.set_strict(strict);

                // It is a Syntax Error if the LexicallyDeclaredNames of ScriptBody contains any duplicate entries.
                // It is a Syntax Error if any element of the LexicallyDeclaredNames of ScriptBody also occurs in the VarDeclaredNames of ScriptBody.
                let mut lexical_names = FxHashSet::default();
                for name in top_level_lexically_declared_names(&statement_list) {
                    if !lexical_names.insert(name) {
                        return Err(Error::general(
                            "lexical name declared multiple times",
                            Position::new(1, 1),
                        ));
                    }
                }

                for name in top_level_var_declared_names(&statement_list) {
                    if lexical_names.contains(&name) {
                        return Err(Error::general(
                            "lexical name declared multiple times",
                            Position::new(1, 1),
                        ));
                    }
                }

                Ok(statement_list)
            }
            None => Ok(StatementList::from(Vec::new())),
        }
    }
}

/// Parses a script body.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ScriptBody
#[derive(Debug, Clone, Copy)]
pub struct ScriptBody {
    direct_eval: bool,
}

impl ScriptBody {
    /// Create a new `ScriptBody` parser.
    #[inline]
    const fn new(direct_eval: bool) -> Self {
        Self { direct_eval }
    }
}

impl<R> TokenParser<R> for ScriptBody
where
    R: Read,
{
    type Output = StatementList;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let body = self::statement::StatementList::new(false, false, false, &[])
            .parse(cursor, interner)?;

        if !self.direct_eval {
            // It is a Syntax Error if StatementList Contains super unless the source text containing super is eval
            // code that is being processed by a direct eval.
            // Additional early error rules for super within direct eval are defined in 19.2.1.1.
            if contains(&body, ContainsSymbol::Super) {
                return Err(Error::general("invalid super usage", Position::new(1, 1)));
            }
            // It is a Syntax Error if StatementList Contains NewTarget unless the source text containing NewTarget
            // is eval code that is being processed by a direct eval.
            // Additional early error rules for NewTarget in direct eval are defined in 19.2.1.1.
            if contains(&body, ContainsSymbol::NewTarget) {
                return Err(Error::general(
                    "invalid new.target usage",
                    Position::new(1, 1),
                ));
            }
        }

        Ok(body)
    }
}

/// Helper to check if any parameter names are declared in the given list.
#[inline]
fn name_in_lexically_declared_names(
    bound_names: &[Identifier],
    lexical_names: &[Identifier],
    position: Position,
) -> ParseResult<()> {
    for name in bound_names {
        if lexical_names.contains(name) {
            return Err(Error::General {
                message: "formal parameter declared in lexically declared names",
                position,
            });
        }
    }
    Ok(())
}

/// Trait to reduce boilerplate in the parser.
trait OrAbrupt<T> {
    /// Will convert an `Ok(None)` to an [`Error::AbruptEnd`] or return the inner type if not.
    fn or_abrupt(self) -> ParseResult<T>;
}

impl<T> OrAbrupt<T> for ParseResult<Option<T>> {
    #[inline]
    fn or_abrupt(self) -> ParseResult<T> {
        self?.ok_or(Error::AbruptEnd)
    }
}
