//! Boa parser implementation.

mod cursor;
mod expression;
mod statement;

pub(crate) mod function;

pub mod error;

#[cfg(test)]
mod tests;

use crate::{
    string::utf16,
    syntax::{
        lexer::TokenKind,
        parser::{
            cursor::Cursor,
            function::{FormalParameters, FunctionStatementList},
        },
    },
    Context, JsString,
};
use boa_interner::Interner;
use rustc_hash::{FxHashMap, FxHashSet};
use std::io::Read;

pub use self::error::{ParseError, ParseResult};

use boa_ast::{
    expression::Identifier, function::FormalParameterList, ContainsSymbol, Position, StatementList,
};

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

#[derive(Debug)]
pub struct Parser<R> {
    /// Cursor of the parser, pointing to the lexer and used to get tokens for the parser.
    cursor: Cursor<R>,
}

impl<R> Parser<R> {
    /// Create a new `Parser` with a reader as the input to parse.
    pub fn new(reader: R) -> Self
    where
        R: Read,
    {
        Self {
            cursor: Cursor::new(reader),
        }
    }

    /// Set the parser strict mode to true.
    pub(crate) fn set_strict(&mut self)
    where
        R: Read,
    {
        self.cursor.set_strict_mode(true);
    }

    /// Parse the full input as a [ECMAScript Script][spec] into the boa AST representation.
    /// The resulting `StatementList` can be compiled into boa bytecode and executed in the boa vm.
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-Script
    pub fn parse_all(&mut self, context: &mut Context) -> Result<StatementList, ParseError>
    where
        R: Read,
    {
        Script::new(false).parse(&mut self.cursor, context)
    }

    /// [`19.2.1.1 PerformEval ( x, strictCaller, direct )`][spec]
    ///
    /// Parses the source text input of an `eval` call.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-performeval
    pub(crate) fn parse_eval(
        &mut self,
        direct: bool,
        context: &mut Context,
    ) -> Result<StatementList, ParseError>
    where
        R: Read,
    {
        #[derive(Debug, Default)]
        #[allow(clippy::struct_excessive_bools)]
        struct Flags {
            in_function: bool,
            in_method: bool,
            in_derived_constructor: bool,
            in_class_field_initializer: bool,
        }
        // 5. Perform ? HostEnsureCanCompileStrings(evalRealm).
        // 11. Perform the following substeps in an implementation-defined order, possibly interleaving parsing and error detection:
        //     a. Let script be ParseText(StringToCodePoints(x), Script).
        //     b. If script is a List of errors, throw a SyntaxError exception.
        //     c. If script Contains ScriptBody is false, return undefined.
        //     d. Let body be the ScriptBody of script.
        let body = Script::new(direct).parse(&mut self.cursor, context)?;

        // 6. Let inFunction be false.
        // 7. Let inMethod be false.
        // 8. Let inDerivedConstructor be false.
        // 9. Let inClassFieldInitializer be false.
        // a. Let thisEnvRec be GetThisEnvironment().
        let flags = match context
            .realm
            .environments
            .get_this_environment()
            .as_function_slots()
        {
            // 10. If direct is true, then
            //     b. If thisEnvRec is a Function Environment Record, then
            Some(function_env) if direct => {
                let function_env = function_env.borrow();
                // i. Let F be thisEnvRec.[[FunctionObject]].
                let function_object = function_env.function_object().borrow();
                Flags {
                    // ii. Set inFunction to true.
                    in_function: true,
                    // iii. Set inMethod to thisEnvRec.HasSuperBinding().
                    in_method: function_env.has_super_binding(),
                    // iv. If F.[[ConstructorKind]] is derived, set inDerivedConstructor to true.
                    in_derived_constructor: function_object
                        .as_function()
                        .expect("must be function object")
                        .is_derived_constructor(),
                    // TODO:
                    // v. Let classFieldInitializerName be F.[[ClassFieldInitializerName]].
                    // vi. If classFieldInitializerName is not empty, set inClassFieldInitializer to true.
                    in_class_field_initializer: false,
                }
            }
            _ => Flags::default(),
        };

        if !flags.in_function && body.contains(ContainsSymbol::NewTarget) {
            return Err(ParseError::general(
                "invalid `new.target` expression inside eval",
                Position::new(1, 1),
            ));
        }
        if !flags.in_method && body.contains(ContainsSymbol::SuperProperty) {
            return Err(ParseError::general(
                "invalid `super` reference inside eval",
                Position::new(1, 1),
            ));
        }
        if !flags.in_derived_constructor && body.contains(ContainsSymbol::SuperCall) {
            return Err(ParseError::general(
                "invalid `super` call inside eval",
                Position::new(1, 1),
            ));
        }
        if flags.in_class_field_initializer && body.contains_arguments() {
            return Err(ParseError::general(
                "invalid `arguments` reference inside eval",
                Position::new(1, 1),
            ));
        }

        Ok(body)
    }

    /// Parse the full input as an [ECMAScript `FunctionBody`][spec] into the boa AST representation.
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-FunctionBody
    pub(crate) fn parse_function_body(
        &mut self,
        interner: &mut Interner,
        allow_yield: bool,
        allow_await: bool,
    ) -> Result<StatementList, ParseError>
    where
        R: Read,
    {
        FunctionStatementList::new(allow_yield, allow_await).parse(&mut self.cursor, interner)
    }

    /// Parse the full input as an [ECMAScript `FormalParameterList`][spec] into the boa AST representation.
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-FormalParameterList
    pub(crate) fn parse_formal_parameters(
        &mut self,
        interner: &mut Interner,
        allow_yield: bool,
        allow_await: bool,
    ) -> Result<FormalParameterList, ParseError>
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
    fn new(direct_eval: bool) -> Self {
        Self { direct_eval }
    }

    fn parse<R: Read>(
        self,
        cursor: &mut Cursor<R>,
        context: &mut Context,
    ) -> Result<StatementList, ParseError> {
        let mut strict = cursor.strict_mode();
        match cursor.peek(0, context.interner_mut())? {
            Some(tok) => {
                match tok.kind() {
                    // Set the strict mode
                    TokenKind::StringLiteral(string)
                        if context.interner_mut().resolve_expect(*string).join(
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
                    ScriptBody::new(self.direct_eval).parse(cursor, context.interner_mut())?;
                statement_list.set_strict(strict);

                // It is a Syntax Error if the LexicallyDeclaredNames of ScriptBody contains any duplicate entries.
                // It is a Syntax Error if any element of the LexicallyDeclaredNames of ScriptBody also occurs in the VarDeclaredNames of ScriptBody.
                let mut var_declared_names = FxHashSet::default();
                statement_list.var_declared_names(&mut var_declared_names);
                let lexically_declared_names = statement_list.lexically_declared_names();
                let mut lexically_declared_names_map: FxHashMap<Identifier, bool> =
                    FxHashMap::default();
                for (name, is_function_declaration) in &lexically_declared_names {
                    if let Some(existing_is_function_declaration) =
                        lexically_declared_names_map.get(name)
                    {
                        if !(*is_function_declaration && *existing_is_function_declaration) {
                            return Err(ParseError::general(
                                "lexical name declared multiple times",
                                Position::new(1, 1),
                            ));
                        }
                    }
                    lexically_declared_names_map.insert(*name, *is_function_declaration);

                    if !is_function_declaration && var_declared_names.contains(name) {
                        return Err(ParseError::general(
                            "lexical name declared in var names",
                            Position::new(1, 1),
                        ));
                    }
                    if context.has_binding(*name) {
                        return Err(ParseError::general(
                            "lexical name declared multiple times",
                            Position::new(1, 1),
                        ));
                    }
                    if !is_function_declaration {
                        let name_str = context.interner().resolve_expect(name.sym());
                        let desc = context
                            .realm
                            .global_property_map
                            .string_property_map()
                            .get(&name_str.into_common::<JsString>(false));
                        let non_configurable_binding_exists = match desc {
                            Some(desc) => !matches!(desc.configurable(), Some(true)),
                            None => false,
                        };
                        if non_configurable_binding_exists {
                            return Err(ParseError::general(
                                "lexical name declared in var names",
                                Position::new(1, 1),
                            ));
                        }
                    }
                }
                for name in var_declared_names {
                    if context.has_binding(name) {
                        return Err(ParseError::general(
                            "lexical name declared in var names",
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
    fn new(direct_eval: bool) -> Self {
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
            for node in body.statements() {
                // It is a Syntax Error if StatementList Contains super unless the source text containing super is eval
                // code that is being processed by a direct eval.
                // Additional early error rules for super within direct eval are defined in 19.2.1.1.
                if node.contains(ContainsSymbol::SuperCall)
                    || node.contains(ContainsSymbol::SuperProperty)
                {
                    return Err(ParseError::general(
                        "invalid super usage",
                        Position::new(1, 1),
                    ));
                }

                // It is a Syntax Error if StatementList Contains NewTarget unless the source text containing NewTarget
                // is eval code that is being processed by a direct eval.
                // Additional early error rules for NewTarget in direct eval are defined in 19.2.1.1.
                if node.contains(ContainsSymbol::NewTarget) {
                    return Err(ParseError::general(
                        "invalid new.target usage",
                        Position::new(1, 1),
                    ));
                }
            }
        }

        Ok(body)
    }
}

// Checks if a function contains a super call or super property access.
fn function_contains_super(body: &StatementList, parameters: &FormalParameterList) -> bool {
    for param in parameters.as_ref() {
        if param.variable().contains(ContainsSymbol::SuperCall)
            || param.variable().contains(ContainsSymbol::SuperProperty)
        {
            return true;
        }
    }
    for node in body.statements() {
        if node.contains(ContainsSymbol::SuperCall) || node.contains(ContainsSymbol::SuperProperty)
        {
            return true;
        }
    }
    false
}

/// Returns `true` if the function parameters or body contain a direct `super` call.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-static-semantics-hasdirectsuper
pub fn has_direct_super(body: &StatementList, parameters: &FormalParameterList) -> bool {
    for param in parameters.as_ref() {
        if param.variable().contains(ContainsSymbol::SuperCall) {
            return true;
        }
    }
    for node in body.statements() {
        if node.contains(ContainsSymbol::SuperCall) {
            return true;
        }
    }
    false
}

/// Helper to check if any parameter names are declared in the given list.
fn name_in_lexically_declared_names(
    parameter_list: &FormalParameterList,
    names: &[Identifier],
    position: Position,
) -> Result<(), ParseError> {
    for parameter in parameter_list.as_ref() {
        for name in &parameter.names() {
            if names.contains(name) {
                return Err(ParseError::General {
                    message: "formal parameter declared in lexically declared names",
                    position,
                });
            }
        }
    }
    Ok(())
}
