//! Boa parser implementation.

mod cursor;
mod expression;
mod statement;

pub(crate) mod function;

#[cfg(test)]
mod tests;

use crate::{
    error::ParseResult,
    lexer::Error as LexError,
    parser::{
        cursor::Cursor,
        function::{FormalParameters, FunctionStatementList},
    },
    Error, Source,
};
use boa_ast::{
    expression::Identifier,
    function::{FormalParameterList, FunctionBody},
    operations::{
        all_private_identifiers_valid, check_labels, contains, contains_invalid_object_literal,
        lexically_declared_names, var_declared_names, ContainsSymbol,
    },
    Position, StatementList,
};
use boa_interner::Interner;
use rustc_hash::FxHashSet;
use std::{io::Read, path::Path};

use self::statement::ModuleItemList;

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
    ///
    /// # Errors
    ///
    /// It will fail if the cursor is not placed at the beginning of the expected non-terminal.
    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output>;
}

/// Boolean representing if the parser should allow a `yield` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowYield(bool);

impl From<bool> for AllowYield {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if the parser should allow a `await` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowAwait(bool);

impl From<bool> for AllowAwait {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if the parser should allow a `in` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowIn(bool);

impl From<bool> for AllowIn {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if the parser should allow a `return` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowReturn(bool);

impl From<bool> for AllowReturn {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if the parser should allow a `default` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowDefault(bool);

impl From<bool> for AllowDefault {
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
pub struct Parser<'a, R> {
    /// Path to the source being parsed.
    #[allow(unused)] // Good to have for future improvements.
    path: Option<&'a Path>,
    /// Cursor of the parser, pointing to the lexer and used to get tokens for the parser.
    cursor: Cursor<R>,
}

impl<'a, R: Read> Parser<'a, R> {
    /// Create a new `Parser` with a `Source` as the input to parse.
    pub fn new(source: Source<'a, R>) -> Self {
        Self {
            path: source.path,
            cursor: Cursor::new(source.reader),
        }
    }

    /// Parse the full input as a [ECMAScript Script][spec] into the boa AST representation.
    /// The resulting `Script` can be compiled into boa bytecode and executed in the boa vm.
    ///
    /// # Errors
    ///
    /// Will return `Err` on any parsing error, including invalid reads of the bytes being parsed.
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-Script
    pub fn parse_script(&mut self, interner: &mut Interner) -> ParseResult<boa_ast::Script> {
        ScriptParser::new(false).parse(&mut self.cursor, interner)
    }

    /// Parse the full input as an [ECMAScript Module][spec] into the boa AST representation.
    /// The resulting `ModuleItemList` can be compiled into boa bytecode and executed in the boa vm.
    ///
    /// # Errors
    ///
    /// Will return `Err` on any parsing error, including invalid reads of the bytes being parsed.
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-Module
    pub fn parse_module(&mut self, interner: &mut Interner) -> ParseResult<boa_ast::Module>
    where
        R: Read,
    {
        ModuleParser.parse(&mut self.cursor, interner)
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
    pub fn parse_eval(
        &mut self,
        direct: bool,
        interner: &mut Interner,
    ) -> ParseResult<boa_ast::Script> {
        ScriptParser::new(direct).parse(&mut self.cursor, interner)
    }

    /// Parses the full input as an [ECMAScript `FunctionBody`][spec] into the boa AST representation.
    ///
    /// # Errors
    ///
    /// Will return `Err` on any parsing error, including invalid reads of the bytes being parsed.
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-FunctionBody
    pub fn parse_function_body(
        &mut self,
        interner: &mut Interner,
        allow_yield: bool,
        allow_await: bool,
    ) -> ParseResult<FunctionBody> {
        FunctionStatementList::new(allow_yield, allow_await).parse(&mut self.cursor, interner)
    }

    /// Parses the full input as an [ECMAScript `FormalParameterList`][spec] into the boa AST representation.
    ///
    /// # Errors
    ///
    /// Will return `Err` on any parsing error, including invalid reads of the bytes being parsed.
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-FormalParameterList
    pub fn parse_formal_parameters(
        &mut self,
        interner: &mut Interner,
        allow_yield: bool,
        allow_await: bool,
    ) -> ParseResult<FormalParameterList> {
        FormalParameters::new(allow_yield, allow_await).parse(&mut self.cursor, interner)
    }
}

impl<R> Parser<'_, R> {
    /// Set the parser strict mode to true.
    pub fn set_strict(&mut self)
    where
        R: Read,
    {
        self.cursor.set_strict(true);
    }

    /// Set the parser JSON mode to true.
    pub fn set_json_parse(&mut self)
    where
        R: Read,
    {
        self.cursor.set_json_parse(true);
    }

    /// Set the unique identifier for the parser.
    pub fn set_identifier(&mut self, identifier: u32)
    where
        R: Read,
    {
        self.cursor.set_identifier(identifier);
    }
}

/// Parses a full script.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-Script
#[derive(Debug, Clone, Copy)]
pub struct ScriptParser {
    direct_eval: bool,
}

impl ScriptParser {
    /// Create a new `Script` parser.
    #[inline]
    const fn new(direct_eval: bool) -> Self {
        Self { direct_eval }
    }
}

impl<R> TokenParser<R> for ScriptParser
where
    R: Read,
{
    type Output = boa_ast::Script;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let script = boa_ast::Script::new(
            ScriptBody::new(true, cursor.strict(), self.direct_eval).parse(cursor, interner)?,
        );

        // It is a Syntax Error if the LexicallyDeclaredNames of ScriptBody contains any duplicate entries.
        let mut lexical_names = FxHashSet::default();
        for name in lexically_declared_names(&script) {
            if !lexical_names.insert(name) {
                return Err(Error::general(
                    "lexical name declared multiple times",
                    Position::new(1, 1),
                ));
            }
        }

        // It is a Syntax Error if any element of the LexicallyDeclaredNames of ScriptBody also occurs in the VarDeclaredNames of ScriptBody.
        for name in var_declared_names(&script) {
            if lexical_names.contains(&name) {
                return Err(Error::general(
                    "lexical name declared multiple times",
                    Position::new(1, 1),
                ));
            }
        }

        Ok(script)
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
    directive_prologues: bool,
    strict: bool,
    direct_eval: bool,
}

impl ScriptBody {
    /// Create a new `ScriptBody` parser.
    #[inline]
    const fn new(directive_prologues: bool, strict: bool, direct_eval: bool) -> Self {
        Self {
            directive_prologues,
            strict,
            direct_eval,
        }
    }
}

impl<R> TokenParser<R> for ScriptBody
where
    R: Read,
{
    type Output = StatementList;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let body = self::statement::StatementList::new(
            false,
            false,
            false,
            &[],
            self.directive_prologues,
            self.strict,
        )
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

            // It is a Syntax Error if AllPrivateIdentifiersValid of StatementList with
            // argument « » is false unless the source text containing ScriptBody is
            // eval code that is being processed by a direct eval.
            if !all_private_identifiers_valid(&body, Vec::new()) {
                return Err(Error::general(
                    "invalid private identifier usage",
                    Position::new(1, 1),
                ));
            }
        }

        if let Err(error) = check_labels(&body) {
            return Err(Error::lex(LexError::Syntax(
                error.message(interner).into(),
                Position::new(1, 1),
            )));
        }

        if contains_invalid_object_literal(&body) {
            return Err(Error::lex(LexError::Syntax(
                "invalid object literal in script statement list".into(),
                Position::new(1, 1),
            )));
        }

        Ok(body)
    }
}

/// Parses a full module.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-Module
#[derive(Debug, Clone, Copy)]
struct ModuleParser;

impl<R> TokenParser<R> for ModuleParser
where
    R: Read,
{
    type Output = boa_ast::Module;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        cursor.set_module();

        let module = boa_ast::Module::new(ModuleItemList.parse(cursor, interner)?);

        // It is a Syntax Error if the LexicallyDeclaredNames of ModuleItemList contains any duplicate entries.
        let mut bindings = FxHashSet::default();
        for name in lexically_declared_names(&module) {
            if !bindings.insert(name) {
                return Err(Error::general(
                    format!(
                        "lexical name `{}` declared multiple times",
                        interner.resolve_expect(name.sym())
                    ),
                    Position::new(1, 1),
                ));
            }
        }

        // It is a Syntax Error if any element of the LexicallyDeclaredNames of ModuleItemList also occurs in the
        // VarDeclaredNames of ModuleItemList.
        for name in var_declared_names(&module) {
            if !bindings.insert(name) {
                return Err(Error::general(
                    format!(
                        "lexical name `{}` declared multiple times",
                        interner.resolve_expect(name.sym())
                    ),
                    Position::new(1, 1),
                ));
            }
        }

        // It is a Syntax Error if the ExportedNames of ModuleItemList contains any duplicate entries.
        {
            let mut exported_names = FxHashSet::default();
            for name in module.items().exported_names() {
                if !exported_names.insert(name) {
                    return Err(Error::general(
                        format!(
                            "exported name `{}` declared multiple times",
                            interner.resolve_expect(name)
                        ),
                        Position::new(1, 1),
                    ));
                }
            }
        }

        // It is a Syntax Error if any element of the ExportedBindings of ModuleItemList does not also occur in either
        // the VarDeclaredNames of ModuleItemList, or the LexicallyDeclaredNames of ModuleItemList.
        for name in module.items().exported_bindings() {
            if !bindings.contains(&name) {
                return Err(Error::general(
                    format!(
                        "could not find the exported binding `{}` in the declared names of the module",
                        interner.resolve_expect(name.sym())
                    ),
                    Position::new(1, 1),
                ));
            }
        }

        // It is a Syntax Error if ModuleItemList Contains super.
        if contains(&module, ContainsSymbol::Super) {
            return Err(Error::general(
                "module cannot contain `super` on the top-level",
                Position::new(1, 1),
            ));
        }

        // It is a Syntax Error if ModuleItemList Contains NewTarget.
        if contains(&module, ContainsSymbol::NewTarget) {
            return Err(Error::general(
                "module cannot contain `new.target` on the top-level",
                Position::new(1, 1),
            ));
        }

        // It is a Syntax Error if ContainsDuplicateLabels of ModuleItemList with argument « » is true.
        // It is a Syntax Error if ContainsUndefinedBreakTarget of ModuleItemList with argument « » is true.
        // It is a Syntax Error if ContainsUndefinedContinueTarget of ModuleItemList with arguments « » and « » is true.
        check_labels(&module).map_err(|error| {
            Error::lex(LexError::Syntax(
                error.message(interner).into(),
                Position::new(1, 1),
            ))
        })?;

        // It is a Syntax Error if AllPrivateIdentifiersValid of ModuleItemList with argument « » is false.
        if !all_private_identifiers_valid(&module, Vec::new()) {
            return Err(Error::general(
                "invalid private identifier usage",
                Position::new(1, 1),
            ));
        }

        Ok(module)
    }
}

/// Helper to check if any parameter names are declared in the given list.
fn name_in_lexically_declared_names(
    bound_names: &[Identifier],
    lexical_names: &[Identifier],
    position: Position,
    interner: &Interner,
) -> ParseResult<()> {
    for name in bound_names {
        if lexical_names.contains(name) {
            return Err(Error::general(
                format!(
                    "formal parameter `{}` declared in lexically declared names",
                    interner.resolve_expect(name.sym())
                ),
                position,
            ));
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
    fn or_abrupt(self) -> ParseResult<T> {
        self?.ok_or(Error::AbruptEnd)
    }
}
