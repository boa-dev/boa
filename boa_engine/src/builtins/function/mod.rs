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
    builtins::{BuiltIn, JsArgs},
    bytecompiler::FunctionCompiler,
    context::intrinsics::StandardConstructors,
    environments::DeclarativeEnvironmentStack,
    error::JsNativeError,
    js_string,
    object::{
        internal_methods::get_prototype_from_constructor, JsObject, NativeObject, Object,
        ObjectData,
    },
    object::{ConstructorBuilder, FunctionBuilder, JsFunction, PrivateElement, Ref, RefMut},
    property::{Attribute, PropertyDescriptor, PropertyKey},
    string::utf16,
    symbol::WellKnownSymbols,
    value::IntegerOrInfinity,
    Context, JsResult, JsString, JsValue,
};
use boa_ast::{
    function::FormalParameterList,
    operations::{bound_names, contains, lexically_declared_names, ContainsSymbol},
    StatementList,
};
use boa_gc::{self, custom_trace, Finalize, Gc, GcCell, Trace};
use boa_interner::Sym;
use boa_parser::Parser;
use boa_profiler::Profiler;
use dyn_clone::DynClone;
use std::{
    any::Any,
    fmt,
    ops::{Deref, DerefMut},
};
use tap::{Conv, Pipe};

use super::promise::PromiseCapability;

pub(crate) mod arguments;
#[cfg(test)]
mod tests;

/// Type representing a native built-in function a.k.a. function pointer.
///
/// Native functions need to have this signature in order to
/// be callable from Javascript.
///
/// # Arguments
///
/// - The first argument represents the `this` variable of every Javascript function.
///
/// - The second argument represents a list of all arguments passed to the function.
///
/// - The last argument is the [`Context`] of the engine.
pub type NativeFunctionSignature = fn(&JsValue, &[JsValue], &mut Context<'_>) -> JsResult<JsValue>;

// Allows restricting closures to only `Copy` ones.
// Used the sealed pattern to disallow external implementations
// of `DynCopy`.
mod sealed {
    pub trait Sealed {}
    impl<T: Copy> Sealed for T {}
}

/// This trait is implemented by any type that implements [`core::marker::Copy`].
pub trait DynCopy: sealed::Sealed {}

impl<T: Copy> DynCopy for T {}

/// Trait representing a native built-in closure.
///
/// Closures need to have this signature in order to
/// be callable from Javascript, but most of the time the compiler
/// is smart enough to correctly infer the types.
pub trait ClosureFunctionSignature:
    Fn(&JsValue, &[JsValue], Captures, &mut Context<'_>) -> JsResult<JsValue>
    + DynCopy
    + DynClone
    + 'static
{
}

impl<T> ClosureFunctionSignature for T where
    T: Fn(&JsValue, &[JsValue], Captures, &mut Context<'_>) -> JsResult<JsValue> + Copy + 'static
{
}

// Allows cloning Box<dyn ClosureFunctionSignature>
dyn_clone::clone_trait_object!(ClosureFunctionSignature);

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
    Private(Sym, JsFunction),
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

/// Wrapper for `Gc<GcCell<dyn NativeObject>>` that allows passing additional
/// captures through a `Copy` closure.
///
/// Any type implementing `Trace + Any + Debug`
/// can be used as a capture context, so you can pass e.g. a String,
/// a tuple or even a full struct.
///
/// You can cast to `Any` with `as_any`, `as_mut_any` and downcast
/// with `Any::downcast_ref` and `Any::downcast_mut` to recover the original
/// type.
#[derive(Clone, Debug, Trace, Finalize)]
pub struct Captures(Gc<GcCell<Box<dyn NativeObject>>>);

impl Captures {
    /// Creates a new capture context.
    pub(crate) fn new<T>(captures: T) -> Self
    where
        T: NativeObject,
    {
        Self(Gc::new(GcCell::new(Box::new(captures))))
    }

    /// Casts `Captures` to `Any`
    ///
    /// # Panics
    ///
    /// Panics if it's already borrowed as `&mut Any`
    pub fn as_any(&self) -> boa_gc::GcCellRef<'_, dyn Any> {
        Ref::map(self.0.borrow(), |data| data.deref().as_any())
    }

    /// Mutably casts `Captures` to `Any`
    ///
    /// # Panics
    ///
    /// Panics if it's already borrowed as `&mut Any`
    pub fn as_mut_any(&self) -> boa_gc::GcCellRefMut<'_, Box<dyn NativeObject>, dyn Any> {
        RefMut::map(self.0.borrow_mut(), |data| data.deref_mut().as_mut_any())
    }
}

/// Boa representation of a Function Object.
///
/// `FunctionBody` is specific to this interpreter, it will either be Rust code or JavaScript code
/// (AST Node).
///
/// <https://tc39.es/ecma262/#sec-ecmascript-function-objects>
#[derive(Finalize)]
pub enum Function {
    /// A rust function.
    Native {
        /// The rust function.
        function: NativeFunctionSignature,

        /// The kind of the function constructor if it is a constructor.
        constructor: Option<ConstructorKind>,
    },

    /// A rust function that may contain captured values.
    Closure {
        /// The rust function.
        function: Box<dyn ClosureFunctionSignature>,

        /// The kind of the function constructor if it is a constructor.
        constructor: Option<ConstructorKind>,

        /// The captured values.
        captures: Captures,
    },

    /// A bytecode function.
    Ordinary {
        /// The code block containing the compiled function.
        code: Gc<crate::vm::CodeBlock>,

        /// The `[[Environment]]` internal slot.
        environments: DeclarativeEnvironmentStack,

        /// The `[[ConstructorKind]]` internal slot.
        constructor_kind: ConstructorKind,

        /// The `[[HomeObject]]` internal slot.
        home_object: Option<JsObject>,

        /// The `[[Fields]]` internal slot.
        fields: Vec<ClassFieldDefinition>,

        /// The `[[PrivateMethods]]` internal slot.
        private_methods: Vec<(Sym, PrivateElement)>,
    },

    /// A bytecode async function.
    Async {
        /// The code block containing the compiled function.
        code: Gc<crate::vm::CodeBlock>,

        /// The `[[Environment]]` internal slot.
        environments: DeclarativeEnvironmentStack,

        /// The `[[HomeObject]]` internal slot.
        home_object: Option<JsObject>,

        /// The promise capability record of the async function.
        promise_capability: PromiseCapability,
    },

    /// A bytecode generator function.
    Generator {
        /// The code block containing the compiled function.
        code: Gc<crate::vm::CodeBlock>,

        /// The `[[Environment]]` internal slot.
        environments: DeclarativeEnvironmentStack,

        /// The `[[HomeObject]]` internal slot.
        home_object: Option<JsObject>,
    },

    /// A bytecode async generator function.
    AsyncGenerator {
        /// The code block containing the compiled function.
        code: Gc<crate::vm::CodeBlock>,

        /// The `[[Environment]]` internal slot.
        environments: DeclarativeEnvironmentStack,

        /// The `[[HomeObject]]` internal slot.
        home_object: Option<JsObject>,
    },
}

unsafe impl Trace for Function {
    custom_trace! {this, {
        match this {
            Self::Native { .. } => {}
            Self::Closure { captures, .. } => mark(captures),
            Self::Ordinary { code, environments, home_object, fields, private_methods, .. } => {
                mark(code);
                mark(environments);
                mark(home_object);
                mark(fields);
                for (_, elem) in private_methods {
                    mark(elem);
                }
            }
            Self::Async { code, environments, home_object, promise_capability } => {
                mark(code);
                mark(environments);
                mark(home_object);
                mark(promise_capability);
            }
            Self::Generator { code, environments, home_object}
            | Self::AsyncGenerator { code, environments, home_object} => {
                mark(code);
                mark(environments);
                mark(home_object);
            }
        }
    }}
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Function {{ ... }}")
    }
}

impl Function {
    /// Returns true if the function object is a constructor.
    pub fn is_constructor(&self) -> bool {
        match self {
            Self::Native { constructor, .. } | Self::Closure { constructor, .. } => {
                constructor.is_some()
            }
            Self::Generator { .. } | Self::AsyncGenerator { .. } | Self::Async { .. } => false,
            Self::Ordinary { code, .. } => !(code.this_mode == ThisMode::Lexical),
        }
    }

    /// Returns true if the function object is a derived constructor.
    pub(crate) const fn is_derived_constructor(&self) -> bool {
        if let Self::Ordinary {
            constructor_kind, ..
        } = self
        {
            constructor_kind.is_derived()
        } else {
            false
        }
    }

    /// Returns a reference to the function `[[HomeObject]]` slot if present.
    pub(crate) const fn get_home_object(&self) -> Option<&JsObject> {
        match self {
            Self::Ordinary { home_object, .. }
            | Self::Async { home_object, .. }
            | Self::Generator { home_object, .. }
            | Self::AsyncGenerator { home_object, .. } => home_object.as_ref(),
            _ => None,
        }
    }

    ///  Sets the `[[HomeObject]]` slot if present.
    pub(crate) fn set_home_object(&mut self, object: JsObject) {
        match self {
            Self::Ordinary { home_object, .. }
            | Self::Async { home_object, .. }
            | Self::Generator { home_object, .. }
            | Self::AsyncGenerator { home_object, .. } => *home_object = Some(object),
            _ => {}
        }
    }

    /// Returns the values of the `[[Fields]]` internal slot.
    pub(crate) fn get_fields(&self) -> &[ClassFieldDefinition] {
        if let Self::Ordinary { fields, .. } = self {
            fields
        } else {
            &[]
        }
    }

    /// Pushes a value to the `[[Fields]]` internal slot if present.
    pub(crate) fn push_field(&mut self, key: PropertyKey, value: JsFunction) {
        if let Self::Ordinary { fields, .. } = self {
            fields.push(ClassFieldDefinition::Public(key, value));
        }
    }

    /// Pushes a private value to the `[[Fields]]` internal slot if present.
    pub(crate) fn push_field_private(&mut self, key: Sym, value: JsFunction) {
        if let Self::Ordinary { fields, .. } = self {
            fields.push(ClassFieldDefinition::Private(key, value));
        }
    }

    /// Returns the values of the `[[PrivateMethods]]` internal slot.
    pub(crate) fn get_private_methods(&self) -> &[(Sym, PrivateElement)] {
        if let Self::Ordinary {
            private_methods, ..
        } = self
        {
            private_methods
        } else {
            &[]
        }
    }

    /// Pushes a private method to the `[[PrivateMethods]]` internal slot if present.
    pub(crate) fn push_private_method(&mut self, name: Sym, method: PrivateElement) {
        if let Self::Ordinary {
            private_methods, ..
        } = self
        {
            private_methods.push((name, method));
        }
    }

    /// Returns the promise capability if the function is an async function.
    pub(crate) const fn get_promise_capability(&self) -> Option<&PromiseCapability> {
        if let Self::Async {
            promise_capability, ..
        } = self
        {
            Some(promise_capability)
        } else {
            None
        }
    }
}

/// Creates a new member function of a `Object` or `prototype`.
///
/// A function registered using this macro can then be called from Javascript using:
///
/// parent.name()
///
/// See the javascript 'Number.toString()' as an example.
///
/// # Arguments
/// function: The function to register as a built in function.
/// name: The name of the function (how it will be called but without the ()).
/// parent: The object to register the function on, if the global object is used then the function is instead called as name()
///     without requiring the parent, see parseInt() as an example.
/// length: As described at <https://tc39.es/ecma262/#sec-function-instances-length>, The value of the "length" property is an integer that
///     indicates the typical number of arguments expected by the function. However, the language permits the function to be invoked with
///     some other number of arguments.
///
/// If no length is provided, the length will be set to 0.
// TODO: deprecate/remove this.
pub(crate) fn make_builtin_fn<N>(
    function: NativeFunctionSignature,
    name: N,
    parent: &JsObject,
    length: usize,
    interpreter: &Context<'_>,
) where
    N: Into<String>,
{
    let name = name.into();
    let _timer = Profiler::global().start_event(&format!("make_builtin_fn: {name}"), "init");

    let function = JsObject::from_proto_and_data(
        interpreter
            .intrinsics()
            .constructors()
            .function()
            .prototype(),
        ObjectData::function(Function::Native {
            function,
            constructor: None,
        }),
    );
    let attribute = PropertyDescriptor::builder()
        .writable(false)
        .enumerable(false)
        .configurable(true);
    function.insert_property("length", attribute.clone().value(length));
    function.insert_property("name", attribute.value(name.as_str()));

    parent.clone().insert_property(
        name,
        PropertyDescriptor::builder()
            .value(function)
            .writable(true)
            .enumerable(false)
            .configurable(true),
    );
}

/// The internal representation of a `Function` object.
#[derive(Debug, Clone, Copy)]
pub struct BuiltInFunctionObject;

impl BuiltInFunctionObject {
    const LENGTH: usize = 1;

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
        Self::create_dynamic_function(new_target, args, false, false, context).map(Into::into)
    }

    /// `CreateDynamicFunction ( constructor, newTarget, kind, args )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-createdynamicfunction
    pub(crate) fn create_dynamic_function(
        new_target: &JsValue,
        args: &[JsValue],
        r#async: bool,
        generator: bool,
        context: &mut Context<'_>,
    ) -> JsResult<JsObject> {
        let default = if r#async && generator {
            StandardConstructors::async_generator_function
        } else if r#async {
            StandardConstructors::async_function
        } else if generator {
            StandardConstructors::generator_function
        } else {
            StandardConstructors::function
        };

        let prototype = get_prototype_from_constructor(new_target, default, context)?;
        if let Some((body_arg, args)) = args.split_last() {
            let parameters = if args.is_empty() {
                FormalParameterList::default()
            } else {
                let mut parameters = Vec::with_capacity(args.len());
                for arg in args {
                    parameters.push(arg.to_string(context)?.as_slice().to_owned());
                }
                let mut parameters = parameters.join(utf16!(","));
                parameters.push(u16::from(b')'));

                // TODO: make parser generic to u32 iterators
                let parameters = match Parser::new(String::from_utf16_lossy(&parameters).as_bytes())
                    .parse_formal_parameters(context.interner_mut(), generator, r#async)
                {
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

            let body_arg = body_arg.to_string(context)?;

            // TODO: make parser generic to u32 iterators
            let body = match Parser::new(body_arg.to_std_string_escaped().as_bytes())
                .parse_function_body(context.interner_mut(), generator, r#async)
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

            let code = FunctionCompiler::new()
                .name(Sym::ANONYMOUS)
                .generator(generator)
                .r#async(r#async)
                .compile(&parameters, &body, context)?;

            let environments = context.realm.environments.pop_to_global();

            let function_object = if generator {
                crate::vm::create_generator_function_object(code, r#async, context)
            } else {
                crate::vm::create_function_object(code, r#async, false, Some(prototype), context)
            };

            context.realm.environments.extend(environments);

            Ok(function_object)
        } else if generator {
            let code = FunctionCompiler::new()
                .name(Sym::ANONYMOUS)
                .generator(true)
                .compile(
                    &FormalParameterList::default(),
                    &StatementList::default(),
                    context,
                )?;

            let environments = context.realm.environments.pop_to_global();
            let function_object =
                crate::vm::create_generator_function_object(code, r#async, context);
            context.realm.environments.extend(environments);

            Ok(function_object)
        } else {
            let code = FunctionCompiler::new().name(Sym::ANONYMOUS).compile(
                &FormalParameterList::default(),
                &StatementList::default(),
                context,
            )?;

            let environments = context.realm.environments.pop_to_global();
            let function_object =
                crate::vm::create_function_object(code, r#async, false, Some(prototype), context);
            context.realm.environments.extend(environments);

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
        if target.has_own_property("length", context)? {
            // a. Let targetLen be ? Get(Target, "length").
            let target_len = target.get("length", context)?;
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
            "length",
            PropertyDescriptor::builder()
                .value(l)
                .writable(false)
                .enumerable(false)
                .configurable(true),
            context,
        )
        .expect("defining the `length` property for a new object should not fail");

        // 8. Let targetName be ? Get(Target, "name").
        let target_name = target.get("name", context)?;

        // 9. If Type(targetName) is not String, set targetName to the empty String.
        let target_name = target_name.as_string().map_or(js_string!(), Clone::clone);

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
        let object = this.as_object().map(JsObject::borrow);
        let function = object
            .as_deref()
            .and_then(Object::as_function)
            .ok_or_else(|| JsNativeError::typ().with_message("Not a function"))?;

        let name = {
            // Is there a case here where if there is no name field on a value
            // name should default to None? Do all functions have names set?
            let value = this
                .as_object()
                .expect("checked that `this` was an object above")
                .get("name", &mut *context)?;
            if value.is_null_or_undefined() {
                None
            } else {
                Some(value.to_string(context)?)
            }
        };

        let name = name
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "anonymous".into());

        match function {
            Function::Native { .. } | Function::Closure { .. } | Function::Ordinary { .. } => {
                Ok(js_string!(utf16!("[Function: "), &name, utf16!("]")).into())
            }
            Function::Async { .. } => {
                Ok(js_string!(utf16!("[AsyncFunction: "), &name, utf16!("]")).into())
            }
            Function::Generator { .. } => {
                Ok(js_string!(utf16!("[GeneratorFunction: "), &name, utf16!("]")).into())
            }
            Function::AsyncGenerator { .. } => {
                Ok(js_string!(utf16!("[AsyncGeneratorFunction: "), &name, utf16!("]")).into())
            }
        }
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

impl BuiltIn for BuiltInFunctionObject {
    const NAME: &'static str = "Function";

    fn init(context: &mut Context<'_>) -> Option<JsValue> {
        let _timer = Profiler::global().start_event("function", "init");

        let function_prototype = context.intrinsics().constructors().function().prototype();
        FunctionBuilder::native(context, Self::prototype)
            .name("")
            .length(0)
            .constructor(false)
            .build_function_prototype(&function_prototype);

        let symbol_has_instance = WellKnownSymbols::has_instance();

        let has_instance = FunctionBuilder::native(context, Self::has_instance)
            .name("[Symbol.iterator]")
            .length(1)
            .constructor(false)
            .build();

        let throw_type_error = context.intrinsics().objects().throw_type_error();

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().function().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .method(Self::apply, "apply", 2)
        .method(Self::bind, "bind", 1)
        .method(Self::call, "call", 1)
        .method(Self::to_string, "toString", 0)
        .property(symbol_has_instance, has_instance, Attribute::default())
        .property_descriptor(
            "caller",
            PropertyDescriptor::builder()
                .get(throw_type_error.clone())
                .set(throw_type_error.clone())
                .enumerable(false)
                .configurable(true),
        )
        .property_descriptor(
            "arguments",
            PropertyDescriptor::builder()
                .get(throw_type_error.clone())
                .set(throw_type_error)
                .enumerable(false)
                .configurable(true),
        )
        .build()
        .conv::<JsValue>()
        .pipe(Some)
    }
}

/// Abstract operation `SetFunctionName`
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-setfunctionname
fn set_function_name(
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
        PropertyKey::Index(index) => js_string!(format!("{index}")),
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
            "name",
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
        Ok(JsObject::from_proto_and_data(
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
    pub const fn this(&self) -> &JsValue {
        &self.this
    }

    /// Get a reference to the bound function's target function.
    pub const fn target_function(&self) -> &JsObject {
        &self.target_function
    }

    /// Get a reference to the bound function's args.
    pub fn args(&self) -> &[JsValue] {
        self.args.as_slice()
    }
}
