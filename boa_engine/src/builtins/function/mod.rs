//! Boa's implementation of ECMAScript's global `Function` object and Native Functions.
//!
//! Objects wrap `Function`s and expose them via call/construct slots.
//!
//! The `Function` object is used for matching text with a pattern.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-function-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function

use crate::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    bytecompiler::FunctionCompiler,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    environments::{EnvironmentStack, PrivateEnvironment},
    error::JsNativeError,
    js_string,
    object::{internal_methods::get_prototype_from_constructor, JsObject, ObjectData},
    object::{JsFunction, PrivateElement, PrivateName},
    property::{Attribute, PropertyDescriptor, PropertyKey},
    realm::Realm,
    string::{common::StaticJsStrings, utf16},
    symbol::JsSymbol,
    value::IntegerOrInfinity,
    vm::{ActiveRunnable, CodeBlock},
    Context, JsArgs, JsResult, JsString, JsValue,
};
use boa_ast::{
    function::{FormalParameterList, FunctionBody},
    operations::{
        all_private_identifiers_valid, bound_names, contains, lexically_declared_names,
        ContainsSymbol,
    },
};
use boa_gc::{self, custom_trace, Finalize, Gc, Trace};
use boa_interner::Sym;
use boa_parser::{Parser, Source};
use boa_profiler::Profiler;
use std::{fmt, io::Read};
use thin_vec::ThinVec;

pub(crate) mod arguments;

#[cfg(test)]
mod tests;

/// Represents the `[[ThisMode]]` internal slot of function objects.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ecmascript-function-objects
#[derive(Debug, Trace, Finalize, PartialEq, Eq, Clone)]
pub enum ThisMode {
    /// The `this` value refers to the `this` value of a lexically enclosing function.
    Lexical,

    /// The `this` value is used exactly as provided by an invocation of the function.
    Strict,

    /// The `this` value of `undefined` or `null` is interpreted as a reference to the global object,
    /// and any other `this` value is first passed to `ToObject`.
    Global,
}

impl ThisMode {
    /// Returns `true` if the this mode is `Lexical`.
    #[must_use]
    pub const fn is_lexical(&self) -> bool {
        matches!(self, Self::Lexical)
    }

    /// Returns `true` if the this mode is `Strict`.
    #[must_use]
    pub const fn is_strict(&self) -> bool {
        matches!(self, Self::Strict)
    }

    /// Returns `true` if the this mode is `Global`.
    #[must_use]
    pub const fn is_global(&self) -> bool {
        matches!(self, Self::Global)
    }
}

/// Represents the `[[ConstructorKind]]` internal slot of function objects.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-ecmascript-function-objects
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstructorKind {
    /// The class constructor is not derived.
    Base,

    /// The class constructor is a derived class constructor.
    Derived,
}

impl ConstructorKind {
    /// Returns `true` if the constructor kind is `Base`.
    #[must_use]
    pub const fn is_base(&self) -> bool {
        matches!(self, Self::Base)
    }

    /// Returns `true` if the constructor kind is `Derived`.
    #[must_use]
    pub const fn is_derived(&self) -> bool {
        matches!(self, Self::Derived)
    }
}

/// Record containing the field definition of classes.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-classfielddefinition-record-specification-type
#[derive(Clone, Debug, Finalize)]
pub enum ClassFieldDefinition {
    /// A class field definition with a `string` or `symbol` as a name.
    Public(PropertyKey, JsFunction),

    /// A class field definition with a private name.
    Private(PrivateName, JsFunction),
}

unsafe impl Trace for ClassFieldDefinition {
    custom_trace! {this, {
        match this {
            Self::Public(_key, func) => {
                mark(func);
            }
            Self::Private(_, func) => {
                mark(func);
            }
        }
    }}
}

#[derive(Finalize)]
pub(crate) enum FunctionKind {
    /// A bytecode function.
    Ordinary {
        /// The `[[ConstructorKind]]` internal slot.
        constructor_kind: ConstructorKind,

        /// The `[[Fields]]` internal slot.
        fields: ThinVec<ClassFieldDefinition>,

        /// The `[[PrivateMethods]]` internal slot.
        private_methods: ThinVec<(PrivateName, PrivateElement)>,
    },

    /// A bytecode async function.
    Async,

    /// A bytecode generator function.
    Generator,

    /// A bytecode async generator function.
    AsyncGenerator,
}

impl fmt::Debug for FunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ordinary { .. } => f
                .debug_struct("FunctionKind::Ordinary")
                .finish_non_exhaustive(),
            Self::Async { .. } => f
                .debug_struct("FunctionKind::Async")
                .finish_non_exhaustive(),
            Self::Generator { .. } => f
                .debug_struct("FunctionKind::Generator")
                .finish_non_exhaustive(),
            Self::AsyncGenerator { .. } => f
                .debug_struct("FunctionKind::AsyncGenerator")
                .finish_non_exhaustive(),
        }
    }
}

unsafe impl Trace for FunctionKind {
    custom_trace! {this, {
        match this {
            Self::Ordinary {
                fields,
                private_methods,
                ..
            } => {
                for elem in fields {
                    mark(elem);
                }
                for (_, elem) in private_methods {
                    mark(elem);
                }
            }
            Self::Async
            | Self::Generator
            | Self::AsyncGenerator => {}
        }
    }}
}

/// Boa representation of a JavaScript Function Object.
///
/// `FunctionBody` is specific to this interpreter, it will either be Rust code or JavaScript code
/// (AST Node).
///
/// <https://tc39.es/ecma262/#sec-ecmascript-function-objects>
#[derive(Debug, Trace, Finalize)]
pub struct OrdinaryFunction {
    /// The code block containing the compiled function.
    pub(crate) code: Gc<CodeBlock>,

    /// The `[[Environment]]` internal slot.
    pub(crate) environments: EnvironmentStack,

    /// The `[[HomeObject]]` internal slot.
    pub(crate) home_object: Option<JsObject>,

    /// The class object that this function is associated with.
    pub(crate) class_object: Option<JsObject>,

    /// The `[[ScriptOrModule]]` internal slot.
    pub(crate) script_or_module: Option<ActiveRunnable>,

    /// The [`Realm`] the function is defined in.
    pub(crate) realm: Realm,

    /// The kind of ordinary function.
    pub(crate) kind: FunctionKind,
}

impl OrdinaryFunction {
    /// Returns the codeblock of the function.
    #[must_use]
    pub fn codeblock(&self) -> &CodeBlock {
        &self.code
    }

    /// Push a private environment to the function.
    pub(crate) fn push_private_environment(&mut self, environment: Gc<PrivateEnvironment>) {
        self.environments.push_private(environment);
    }

    /// Returns true if the function object is a derived constructor.
    pub(crate) const fn is_derived_constructor(&self) -> bool {
        if let FunctionKind::Ordinary {
            constructor_kind, ..
        } = self.kind
        {
            constructor_kind.is_derived()
        } else {
            false
        }
    }

    /// Does this function have the `[[ClassFieldInitializerName]]` internal slot set to non-empty value.
    pub(crate) fn in_class_field_initializer(&self) -> bool {
        self.code.in_class_field_initializer()
    }

    /// Returns a reference to the function `[[HomeObject]]` slot if present.
    pub(crate) const fn get_home_object(&self) -> Option<&JsObject> {
        self.home_object.as_ref()
    }

    ///  Sets the `[[HomeObject]]` slot if present.
    pub(crate) fn set_home_object(&mut self, object: JsObject) {
        self.home_object = Some(object);
    }

    /// Returns the values of the `[[Fields]]` internal slot.
    pub(crate) fn get_fields(&self) -> &[ClassFieldDefinition] {
        if let FunctionKind::Ordinary { fields, .. } = &self.kind {
            fields
        } else {
            &[]
        }
    }

    /// Pushes a value to the `[[Fields]]` internal slot if present.
    pub(crate) fn push_field(&mut self, key: PropertyKey, value: JsFunction) {
        if let FunctionKind::Ordinary { fields, .. } = &mut self.kind {
            fields.push(ClassFieldDefinition::Public(key, value));
        }
    }

    /// Pushes a private value to the `[[Fields]]` internal slot if present.
    pub(crate) fn push_field_private(&mut self, name: PrivateName, value: JsFunction) {
        if let FunctionKind::Ordinary { fields, .. } = &mut self.kind {
            fields.push(ClassFieldDefinition::Private(name, value));
        }
    }

    /// Returns the values of the `[[PrivateMethods]]` internal slot.
    pub(crate) fn get_private_methods(&self) -> &[(PrivateName, PrivateElement)] {
        if let FunctionKind::Ordinary {
            private_methods, ..
        } = &self.kind
        {
            private_methods
        } else {
            &[]
        }
    }

    /// Pushes a private method to the `[[PrivateMethods]]` internal slot if present.
    pub(crate) fn push_private_method(&mut self, name: PrivateName, method: PrivateElement) {
        if let FunctionKind::Ordinary {
            private_methods, ..
        } = &mut self.kind
        {
            private_methods.push((name, method));
        }
    }

    ///  Sets the class object.
    pub(crate) fn set_class_object(&mut self, object: JsObject) {
        self.class_object = Some(object);
    }

    /// Gets the `Realm` from where this function originates.
    #[must_use]
    pub const fn realm(&self) -> &Realm {
        &self.realm
    }

    /// Gets a reference to the [`FunctionKind`] of the `Function`.
    pub(crate) const fn kind(&self) -> &FunctionKind {
        &self.kind
    }

    /// Gets a mutable reference to the [`FunctionKind`] of the `Function`.
    pub(crate) fn kind_mut(&mut self) -> &mut FunctionKind {
        &mut self.kind
    }
}

/// The internal representation of a `Function` object.
#[derive(Debug, Clone, Copy)]
pub struct BuiltInFunctionObject;

impl IntrinsicObject for BuiltInFunctionObject {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(std::any::type_name::<Self>(), "init");

        let has_instance = BuiltInBuilder::callable(realm, Self::has_instance)
            .name(js_string!("[Symbol.hasInstance]"))
            .length(1)
            .build();

        let throw_type_error = realm.intrinsics().objects().throw_type_error();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::apply, js_string!("apply"), 2)
            .method(Self::bind, js_string!("bind"), 1)
            .method(Self::call, js_string!("call"), 1)
            .method(Self::to_string, js_string!("toString"), 0)
            .property(JsSymbol::has_instance(), has_instance, Attribute::default())
            .accessor(
                utf16!("caller"),
                Some(throw_type_error.clone()),
                Some(throw_type_error.clone()),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                utf16!("arguments"),
                Some(throw_type_error.clone()),
                Some(throw_type_error),
                Attribute::CONFIGURABLE,
            )
            .build();

        let prototype = realm.intrinsics().constructors().function().prototype();

        BuiltInBuilder::callable_with_object(realm, prototype.clone(), Self::prototype)
            .name(js_string!())
            .length(0)
            .build();

        prototype.set_prototype(Some(realm.intrinsics().constructors().object().prototype()));
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for BuiltInFunctionObject {
    const NAME: JsString = StaticJsStrings::FUNCTION;
}

impl BuiltInConstructor for BuiltInFunctionObject {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::function;

    /// `Function ( p1, p2, … , pn, body )`
    ///
    /// The apply() method invokes self with the first argument as the `this` value
    /// and the rest of the arguments provided as an array (or an array-like object).
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function-p1-p2-pn-body
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/Function
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let active_function = context
            .active_function_object()
            .unwrap_or_else(|| context.intrinsics().constructors().function().constructor());
        Self::create_dynamic_function(active_function, new_target, args, false, false, context)
            .map(Into::into)
    }
}

impl BuiltInFunctionObject {
    /// `CreateDynamicFunction ( constructor, newTarget, kind, args )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createdynamicfunction
    pub(crate) fn create_dynamic_function(
        constructor: JsObject,
        new_target: &JsValue,
        args: &[JsValue],
        r#async: bool,
        generator: bool,
        context: &mut Context<'_>,
    ) -> JsResult<JsObject> {
        // 1. Let currentRealm be the current Realm Record.
        // 2. Perform ? HostEnsureCanCompileStrings(currentRealm).
        context
            .host_hooks()
            .ensure_can_compile_strings(context.realm().clone(), context)?;

        // 3. If newTarget is undefined, set newTarget to constructor.
        let new_target = if new_target.is_undefined() {
            constructor.into()
        } else {
            new_target.clone()
        };

        let default = if r#async && generator {
            // 7. Else,
            //     a. Assert: kind is asyncGenerator.
            //     b. Let prefix be "async function*".
            //     c. Let exprSym be the grammar symbol AsyncGeneratorExpression.
            //     d. Let bodySym be the grammar symbol AsyncGeneratorBody.
            //     e. Let parameterSym be the grammar symbol FormalParameters[+Yield, +Await].
            //     f. Let fallbackProto be "%AsyncGeneratorFunction.prototype%".
            StandardConstructors::async_generator_function
        } else if r#async {
            // 6. Else if kind is async, then
            //     a. Let prefix be "async function".
            //     b. Let exprSym be the grammar symbol AsyncFunctionExpression.
            //     c. Let bodySym be the grammar symbol AsyncFunctionBody.
            //     d. Let parameterSym be the grammar symbol FormalParameters[~Yield, +Await].
            //     e. Let fallbackProto be "%AsyncFunction.prototype%".
            StandardConstructors::async_function
        } else if generator {
            // 5. Else if kind is generator, then
            //     a. Let prefix be "function*".
            //     b. Let exprSym be the grammar symbol GeneratorExpression.
            //     c. Let bodySym be the grammar symbol GeneratorBody.
            //     d. Let parameterSym be the grammar symbol FormalParameters[+Yield, ~Await].
            //     e. Let fallbackProto be "%GeneratorFunction.prototype%".
            StandardConstructors::generator_function
        } else {
            // 4. If kind is normal, then
            //     a. Let prefix be "function".
            //     b. Let exprSym be the grammar symbol FunctionExpression.
            //     c. Let bodySym be the grammar symbol FunctionBody[~Yield, ~Await].
            //     d. Let parameterSym be the grammar symbol FormalParameters[~Yield, ~Await].
            //     e. Let fallbackProto be "%Function.prototype%".
            StandardConstructors::function
        };

        // 22. Let proto be ? GetPrototypeFromConstructor(newTarget, fallbackProto).
        let prototype = get_prototype_from_constructor(&new_target, default, context)?;

        if let Some((body_arg, args)) = args.split_last() {
            let parameters = if args.is_empty() {
                FormalParameterList::default()
            } else {
                let mut parameters = Vec::with_capacity(args.len());
                for arg in args {
                    parameters.push(arg.to_string(context)?);
                }
                let parameters = parameters.join(utf16!(","));

                // TODO: make parser generic to u32 iterators
                let parameters = String::from_utf16_lossy(&parameters);
                let mut parser = Parser::new(Source::from_bytes(&parameters));
                parser.set_identifier(context.next_parser_identifier());

                let parameters = match parser.parse_formal_parameters(
                    context.interner_mut(),
                    generator,
                    r#async,
                ) {
                    Ok(parameters) => parameters,
                    Err(e) => {
                        return Err(JsNativeError::syntax()
                            .with_message(format!("failed to parse function parameters: {e}"))
                            .into())
                    }
                };

                if generator && contains(&parameters, ContainsSymbol::YieldExpression) {
                    return Err(JsNativeError::syntax().with_message(
                            "yield expression is not allowed in formal parameter list of generator function",
                        ).into());
                }

                parameters
            };

            // It is a Syntax Error if FormalParameters Contains YieldExpression is true.
            if generator && r#async && contains(&parameters, ContainsSymbol::YieldExpression) {
                return Err(JsNativeError::syntax()
                    .with_message("yield expression not allowed in async generator parameters")
                    .into());
            }

            // It is a Syntax Error if FormalParameters Contains AwaitExpression is true.
            if generator && r#async && contains(&parameters, ContainsSymbol::AwaitExpression) {
                return Err(JsNativeError::syntax()
                    .with_message("await expression not allowed in async generator parameters")
                    .into());
            }

            // 11. Let bodyString be the string-concatenation of 0x000A (LINE FEED), ? ToString(bodyArg), and 0x000A (LINE FEED).
            let body_arg = body_arg.to_string(context)?.to_std_string_escaped();
            let body = b"\n".chain(body_arg.as_bytes()).chain(b"\n".as_slice());

            // TODO: make parser generic to u32 iterators
            let mut parser = Parser::new(Source::from_reader(body, None));
            parser.set_identifier(context.next_parser_identifier());

            let body = match parser.parse_function_body(context.interner_mut(), generator, r#async)
            {
                Ok(statement_list) => statement_list,
                Err(e) => {
                    return Err(JsNativeError::syntax()
                        .with_message(format!("failed to parse function body: {e}"))
                        .into())
                }
            };

            // Early Error: If BindingIdentifier is present and the source text matched by BindingIdentifier is strict mode code,
            // it is a Syntax Error if the StringValue of BindingIdentifier is "eval" or "arguments".
            if body.strict() {
                for name in bound_names(&parameters) {
                    if name == Sym::ARGUMENTS || name == Sym::EVAL {
                        return Err(JsNativeError::syntax()
                            .with_message(" Unexpected 'eval' or 'arguments' in strict mode")
                            .into());
                    }
                }
            }

            // Early Error: If the source code matching FormalParameters is strict mode code,
            // the Early Error rules for UniqueFormalParameters : FormalParameters are applied.
            if (body.strict()) && parameters.has_duplicates() {
                return Err(JsNativeError::syntax()
                    .with_message("Duplicate parameter name not allowed in this context")
                    .into());
            }

            // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of GeneratorBody is true
            // and IsSimpleParameterList of FormalParameters is false.
            if body.strict() && !parameters.is_simple() {
                return Err(JsNativeError::syntax()
                    .with_message(
                        "Illegal 'use strict' directive in function with non-simple parameter list",
                    )
                    .into());
            }

            // It is a Syntax Error if FunctionBody Contains SuperProperty is true.
            if contains(&body, ContainsSymbol::SuperProperty) {
                return Err(JsNativeError::syntax()
                    .with_message("invalid `super` reference")
                    .into());
            }

            // It is a Syntax Error if FunctionBody Contains SuperCall is true.
            if contains(&body, ContainsSymbol::SuperCall) {
                return Err(JsNativeError::syntax()
                    .with_message("invalid `super` call")
                    .into());
            }

            // It is a Syntax Error if any element of the BoundNames of FormalParameters
            // also occurs in the LexicallyDeclaredNames of FunctionBody.
            // https://tc39.es/ecma262/#sec-function-definitions-static-semantics-early-errors
            {
                let lexically_declared_names = lexically_declared_names(&body);
                for name in bound_names(&parameters) {
                    if lexically_declared_names.contains(&name) {
                        return Err(JsNativeError::syntax()
                            .with_message(format!(
                                "Redeclaration of formal parameter `{}`",
                                context.interner().resolve_expect(name.sym())
                            ))
                            .into());
                    }
                }
            }

            if !all_private_identifiers_valid(&parameters, Vec::new()) {
                return Err(JsNativeError::syntax()
                    .with_message("invalid private identifier usage")
                    .into());
            }

            if !all_private_identifiers_valid(&body, Vec::new()) {
                return Err(JsNativeError::syntax()
                    .with_message("invalid private identifier usage")
                    .into());
            }

            let code = FunctionCompiler::new()
                .name(Sym::ANONYMOUS)
                .generator(generator)
                .r#async(r#async)
                .compile(
                    &parameters,
                    &body,
                    context.realm().environment().compile_env(),
                    context,
                );

            let environments = context.vm.environments.pop_to_global();

            let function_object = if generator {
                crate::vm::create_generator_function_object(code, r#async, Some(prototype), context)
            } else {
                crate::vm::create_function_object(code, r#async, prototype, context)
            };

            context.vm.environments.extend(environments);

            Ok(function_object)
        } else if generator {
            let code = FunctionCompiler::new()
                .name(Sym::ANONYMOUS)
                .generator(true)
                .r#async(r#async)
                .compile(
                    &FormalParameterList::default(),
                    &FunctionBody::default(),
                    context.realm().environment().compile_env(),
                    context,
                );

            let environments = context.vm.environments.pop_to_global();
            let function_object = crate::vm::create_generator_function_object(
                code,
                r#async,
                Some(prototype),
                context,
            );
            context.vm.environments.extend(environments);

            Ok(function_object)
        } else {
            let code = FunctionCompiler::new()
                .r#async(r#async)
                .name(Sym::ANONYMOUS)
                .compile(
                    &FormalParameterList::default(),
                    &FunctionBody::default(),
                    context.realm().environment().compile_env(),
                    context,
                );

            let environments = context.vm.environments.pop_to_global();
            let function_object =
                crate::vm::create_function_object(code, r#async, prototype, context);
            context.vm.environments.extend(environments);

            Ok(function_object)
        }
    }

    /// `Function.prototype.apply ( thisArg, argArray )`
    ///
    /// The apply() method invokes self with the first argument as the `this` value
    /// and the rest of the arguments provided as an array (or an array-like object).
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function.prototype.apply
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/apply
    fn apply(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let func be the this value.
        // 2. If IsCallable(func) is false, throw a TypeError exception.
        let func = this.as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message(format!("{} is not a function", this.display()))
        })?;

        let this_arg = args.get_or_undefined(0);
        let arg_array = args.get_or_undefined(1);
        // 3. If argArray is undefined or null, then
        if arg_array.is_null_or_undefined() {
            // a. Perform PrepareForTailCall().
            // TODO?: 3.a. PrepareForTailCall

            // b. Return ? Call(func, thisArg).
            return func.call(this_arg, &[], context);
        }

        // 4. Let argList be ? CreateListFromArrayLike(argArray).
        let arg_list = arg_array.create_list_from_array_like(&[], context)?;

        // 5. Perform PrepareForTailCall().
        // TODO?: 5. PrepareForTailCall

        // 6. Return ? Call(func, thisArg, argList).
        func.call(this_arg, &arg_list, context)
    }

    /// `Function.prototype.bind ( thisArg, ...args )`
    ///
    /// The bind() method creates a new function that, when called, has its
    /// this keyword set to the provided value, with a given sequence of arguments
    /// preceding any provided when the new function is called.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function.prototype.bind
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_objects/Function/bind
    fn bind(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let Target be the this value.
        // 2. If IsCallable(Target) is false, throw a TypeError exception.
        let target = this.as_callable().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("cannot bind `this` without a `[[Call]]` internal method")
        })?;

        let this_arg = args.get_or_undefined(0).clone();
        let bound_args = args.get(1..).unwrap_or(&[]).to_vec();
        let arg_count = bound_args.len() as i64;

        // 3. Let F be ? BoundFunctionCreate(Target, thisArg, args).
        let f = BoundFunction::create(target.clone(), this_arg, bound_args, context)?;

        // 4. Let L be 0.
        let mut l = JsValue::new(0);

        // 5. Let targetHasLength be ? HasOwnProperty(Target, "length").
        // 6. If targetHasLength is true, then
        if target.has_own_property(utf16!("length"), context)? {
            // a. Let targetLen be ? Get(Target, "length").
            let target_len = target.get(utf16!("length"), context)?;
            // b. If Type(targetLen) is Number, then
            if target_len.is_number() {
                // 1. Let targetLenAsInt be ! ToIntegerOrInfinity(targetLen).
                match target_len
                    .to_integer_or_infinity(context)
                    .expect("to_integer_or_infinity cannot fail for a number")
                {
                    // i. If targetLen is +∞𝔽, set L to +∞.
                    IntegerOrInfinity::PositiveInfinity => l = f64::INFINITY.into(),
                    // ii. Else if targetLen is -∞𝔽, set L to 0.
                    IntegerOrInfinity::NegativeInfinity => {}
                    // iii. Else,
                    IntegerOrInfinity::Integer(target_len) => {
                        // 2. Assert: targetLenAsInt is finite.
                        // 3. Let argCount be the number of elements in args.
                        // 4. Set L to max(targetLenAsInt - argCount, 0).
                        l = (target_len - arg_count).max(0).into();
                    }
                }
            }
        }

        // 7. Perform ! SetFunctionLength(F, L).
        f.define_property_or_throw(
            utf16!("length"),
            PropertyDescriptor::builder()
                .value(l)
                .writable(false)
                .enumerable(false)
                .configurable(true),
            context,
        )
        .expect("defining the `length` property for a new object should not fail");

        // 8. Let targetName be ? Get(Target, "name").
        let target_name = target.get(utf16!("name"), context)?;

        // 9. If Type(targetName) is not String, set targetName to the empty String.
        let target_name = target_name
            .as_string()
            .map_or_else(JsString::default, Clone::clone);

        // 10. Perform SetFunctionName(F, targetName, "bound").
        set_function_name(&f, &target_name.into(), Some(js_string!("bound")), context);

        // 11. Return F.
        Ok(f.into())
    }

    /// `Function.prototype.call ( thisArg, ...args )`
    ///
    /// The call() method calls a function with a given this value and arguments provided individually.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function.prototype.call
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/call
    fn call(this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let func be the this value.
        // 2. If IsCallable(func) is false, throw a TypeError exception.
        let func = this.as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message(format!("{} is not a function", this.display()))
        })?;
        let this_arg = args.get_or_undefined(0);

        // 3. Perform PrepareForTailCall().
        // TODO?: 3. Perform PrepareForTailCall

        // 4. Return ? Call(func, thisArg, args).
        func.call(this_arg, args.get(1..).unwrap_or(&[]), context)
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_string(this: &JsValue, _: &[JsValue], context: &mut Context<'_>) -> JsResult<JsValue> {
        // 1. Let func be the this value.
        let func = this;

        // 2. If func is an Object, func has a [[SourceText]] internal slot, func.[[SourceText]] is a sequence of Unicode code points,and HostHasSourceTextAvailable(func) is true, then
        //     a. Return CodePointsToString(func.[[SourceText]]).

        // 3. If func is a built-in function object, return an implementation-defined String source code representation of func.
        //    The representation must have the syntax of a NativeFunction. Additionally, if func has an [[InitialName]] internal slot and
        //    func.[[InitialName]] is a String, the portion of the returned String that would be matched by
        //    NativeFunctionAccessor_opt PropertyName must be the value of func.[[InitialName]].

        // 4. If func is an Object and IsCallable(func) is true, return an implementation-defined String source code representation of func.
        //    The representation must have the syntax of a NativeFunction.
        // 5. Throw a TypeError exception.
        let Some(object) = func.as_object() else {
            return Err(JsNativeError::typ().with_message("not a function").into());
        };

        if object.borrow().is_native_function() {
            let name = {
                // Is there a case here where if there is no name field on a value
                // name should default to None? Do all functions have names set?
                let value = this
                    .as_object()
                    .expect("checked that `this` was an object above")
                    .get(utf16!("name"), &mut *context)?;
                if value.is_null_or_undefined() {
                    js_string!()
                } else {
                    value.to_string(context)?
                }
            };
            return Ok(
                js_string!(utf16!("function "), &name, utf16!("() { [native code] }")).into(),
            );
        }

        let object = object.borrow();
        let function = object
            .as_function()
            .ok_or_else(|| JsNativeError::typ().with_message("not a function"))?;

        let code = function.codeblock();

        let prefix = match function.kind {
            FunctionKind::Ordinary { .. } => {
                utf16!("function ")
            }
            FunctionKind::Async { .. } => {
                utf16!("async function ")
            }
            FunctionKind::Generator { .. } => {
                utf16!("function* ")
            }
            FunctionKind::AsyncGenerator { .. } => utf16!("async function* "),
        };

        Ok(js_string!(prefix, code.name(), utf16!("() { [native code] }")).into())
    }

    /// `Function.prototype [ @@hasInstance ] ( V )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function.prototype-@@hasinstance
    fn has_instance(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let F be the this value.
        // 2. Return ? OrdinaryHasInstance(F, V).
        Ok(JsValue::ordinary_has_instance(this, args.get_or_undefined(0), context)?.into())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn prototype(_: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        Ok(JsValue::undefined())
    }
}

/// Abstract operation `SetFunctionName`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-setfunctionname
pub(crate) fn set_function_name(
    function: &JsObject,
    name: &PropertyKey,
    prefix: Option<JsString>,
    context: &mut Context<'_>,
) {
    // 1. Assert: F is an extensible object that does not have a "name" own property.
    // 2. If Type(name) is Symbol, then
    let mut name = match name {
        PropertyKey::Symbol(sym) => {
            // a. Let description be name's [[Description]] value.
            // b. If description is undefined, set name to the empty String.
            // c. Else, set name to the string-concatenation of "[", description, and "]".
            sym.description().map_or_else(
                || js_string!(),
                |desc| js_string!(utf16!("["), &desc, utf16!("]")),
            )
        }
        PropertyKey::String(string) => string.clone(),
        PropertyKey::Index(index) => js_string!(format!("{}", index.get())),
    };

    // 3. Else if name is a Private Name, then
    // a. Set name to name.[[Description]].
    // todo: implement Private Names

    // 4. If F has an [[InitialName]] internal slot, then
    // a. Set F.[[InitialName]] to name.
    // todo: implement [[InitialName]] for builtins

    // 5. If prefix is present, then
    if let Some(prefix) = prefix {
        name = js_string!(&prefix, utf16!(" "), &name);
        // b. If F has an [[InitialName]] internal slot, then
        // i. Optionally, set F.[[InitialName]] to name.
        // todo: implement [[InitialName]] for builtins
    }

    // 6. Return ! DefinePropertyOrThrow(F, "name", PropertyDescriptor { [[Value]]: name,
    // [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }).
    function
        .define_property_or_throw(
            utf16!("name"),
            PropertyDescriptor::builder()
                .value(name)
                .writable(false)
                .enumerable(false)
                .configurable(true),
            context,
        )
        .expect("defining the `name` property must not fail per the spec");
}

/// Binds a `Function Object` when `bind` is called.
#[derive(Debug, Trace, Finalize)]
pub struct BoundFunction {
    target_function: JsObject,
    this: JsValue,
    args: Vec<JsValue>,
}

impl BoundFunction {
    /// Abstract operation `BoundFunctionCreate`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-boundfunctioncreate
    pub fn create(
        target_function: JsObject,
        this: JsValue,
        args: Vec<JsValue>,
        context: &mut Context<'_>,
    ) -> JsResult<JsObject> {
        // 1. Let proto be ? targetFunction.[[GetPrototypeOf]]().
        let proto = target_function.__get_prototype_of__(context)?;
        let is_constructor = target_function.is_constructor();

        // 2. Let internalSlotsList be the internal slots listed in Table 35, plus [[Prototype]] and [[Extensible]].
        // 3. Let obj be ! MakeBasicObject(internalSlotsList).
        // 4. Set obj.[[Prototype]] to proto.
        // 5. Set obj.[[Call]] as described in 10.4.1.1.
        // 6. If IsConstructor(targetFunction) is true, then
        // a. Set obj.[[Construct]] as described in 10.4.1.2.
        // 7. Set obj.[[BoundTargetFunction]] to targetFunction.
        // 8. Set obj.[[BoundThis]] to boundThis.
        // 9. Set obj.[[BoundArguments]] to boundArgs.
        // 10. Return obj.
        Ok(JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            ObjectData::bound_function(
                Self {
                    target_function,
                    this,
                    args,
                },
                is_constructor,
            ),
        ))
    }

    /// Get a reference to the bound function's this.
    #[must_use]
    pub const fn this(&self) -> &JsValue {
        &self.this
    }

    /// Get a reference to the bound function's target function.
    #[must_use]
    pub const fn target_function(&self) -> &JsObject {
        &self.target_function
    }

    /// Get a reference to the bound function's args.
    #[must_use]
    pub fn args(&self) -> &[JsValue] {
        self.args.as_slice()
    }
}
