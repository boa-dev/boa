//! This module implements the global `Function` object as well as creates Native Functions.
//!
//! Objects wrap `Function`s and expose them via call/construct slots.
//!
//! `The `Function` object is used for matching text with a pattern.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-function-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function

use std::{
    fmt,
    ops::{Deref, DerefMut},
    option::Option,
};

use dyn_clone::DynClone;

use crate::{
    builtins::BuiltIn,
    context::StandardObjects,
    environment::lexical_environment::Environment,
    gc::{Finalize, Trace},
    object::JsObject,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, FunctionBuilder,
        NativeObject, ObjectData,
    },
    property::Attribute,
    property::PropertyDescriptor,
    syntax::ast::node::{FormalParameter, RcStatementList},
    syntax::ast::node::declaration::Declaration,
    BoaProfiler, Context, JsResult, JsValue,
};

use super::JsArgs;

pub(crate) mod arguments;
#[cfg(test)]
mod tests;

/// Type representing a native built-in function a.k.a. function pointer.
///
/// Native functions need to have this signature in order to
/// be callable from Javascript.
pub type NativeFunctionSignature = fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue>;

// Allows restricting closures to only `Copy` ones.
// Used the sealed pattern to disallow external implementations
// of `DynCopy`.
mod sealed {
    pub trait Sealed {}
    impl<T: Copy> Sealed for T {}
}
pub trait DynCopy: sealed::Sealed {}
impl<T: Copy> DynCopy for T {}

/// Trait representing a native built-in closure.
///
/// Closures need to have this signature in order to
/// be callable from Javascript, but most of the time the compiler
/// is smart enough to correctly infer the types.
pub trait ClosureFunctionSignature:
    Fn(&JsValue, &[JsValue], Captures, &mut Context) -> JsResult<JsValue> + DynCopy + DynClone + 'static
{
}

// The `Copy` bound automatically infers `DynCopy` and `DynClone`
impl<T> ClosureFunctionSignature for T where
    T: Fn(&JsValue, &[JsValue], Captures, &mut Context) -> JsResult<JsValue> + Copy + 'static
{
}

// Allows cloning Box<dyn ClosureFunctionSignature>
dyn_clone::clone_trait_object!(ClosureFunctionSignature);

#[derive(Debug, Trace, Finalize, PartialEq, Clone)]
pub enum ThisMode {
    Lexical,
    Strict,
    Global,
}

impl ThisMode {
    /// Returns `true` if the this mode is `Lexical`.
    pub fn is_lexical(&self) -> bool {
        matches!(self, Self::Lexical)
    }

    /// Returns `true` if the this mode is `Strict`.
    pub fn is_strict(&self) -> bool {
        matches!(self, Self::Strict)
    }

    /// Returns `true` if the this mode is `Global`.
    pub fn is_global(&self) -> bool {
        matches!(self, Self::Global)
    }
}

#[derive(Debug, Trace, Finalize, PartialEq, Clone)]
pub enum ConstructorKind {
    Base,
    Derived,
}

impl ConstructorKind {
    /// Returns `true` if the constructor kind is `Base`.
    pub fn is_base(&self) -> bool {
        matches!(self, Self::Base)
    }

    /// Returns `true` if the constructor kind is `Derived`.
    pub fn is_derived(&self) -> bool {
        matches!(self, Self::Derived)
    }
}
// We don't use a standalone `NativeObject` for `Captures` because it doesn't
// guarantee that the internal type implements `Clone`.
// This private trait guarantees that the internal type passed to `Captures`
// implements `Clone`, and `DynClone` allows us to implement `Clone` for
// `Box<dyn CapturesObject>`.
trait CapturesObject: NativeObject + DynClone {}
impl<T: NativeObject + Clone> CapturesObject for T {}
dyn_clone::clone_trait_object!(CapturesObject);

/// Wrapper for `Box<dyn NativeObject + Clone>` that allows passing additional
/// captures through a `Copy` closure.
///
/// Any type implementing `Trace + Any + Debug + Clone`
/// can be used as a capture context, so you can pass e.g. a String,
/// a tuple or even a full struct.
///
/// You can downcast to any type and handle the fail case as you like
/// with `downcast_ref` and `downcast_mut`, or you can use `try_downcast_ref`
/// and `try_downcast_mut` to automatically throw a `TypeError` if the downcast
/// fails.
#[derive(Debug, Clone, Trace, Finalize)]
pub struct Captures(Box<dyn CapturesObject>);

impl Captures {
    /// Creates a new capture context.
    pub(crate) fn new<T>(captures: T) -> Self
    where
        T: NativeObject + Clone,
    {
        Self(Box::new(captures))
    }

    /// Downcasts `Captures` to the specified type, returning a reference to the
    /// downcasted type if successful or `None` otherwise.
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: NativeObject + Clone,
    {
        self.0.deref().as_any().downcast_ref::<T>()
    }

    /// Mutably downcasts `Captures` to the specified type, returning a
    /// mutable reference to the downcasted type if successful or `None` otherwise.
    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: NativeObject + Clone,
    {
        self.0.deref_mut().as_mut_any().downcast_mut::<T>()
    }

    /// Downcasts `Captures` to the specified type, returning a reference to the
    /// downcasted type if successful or a `TypeError` otherwise.
    pub fn try_downcast_ref<T>(&self, context: &mut Context) -> JsResult<&T>
    where
        T: NativeObject + Clone,
    {
        self.0
            .deref()
            .as_any()
            .downcast_ref::<T>()
            .ok_or_else(|| context.construct_type_error("cannot downcast `Captures` to given type"))
    }

    /// Downcasts `Captures` to the specified type, returning a reference to the
    /// downcasted type if successful or a `TypeError` otherwise.
    pub fn try_downcast_mut<T>(&mut self, context: &mut Context) -> JsResult<&mut T>
    where
        T: NativeObject + Clone,
    {
        self.0
            .deref_mut()
            .as_mut_any()
            .downcast_mut::<T>()
            .ok_or_else(|| context.construct_type_error("cannot downcast `Captures` to given type"))
    }
}

/// Boa representation of a Function Object.
///
/// FunctionBody is specific to this interpreter, it will either be Rust code or JavaScript code (AST Node)
///
/// <https://tc39.es/ecma262/#sec-ecmascript-function-objects>
#[derive(Clone, Trace, Finalize)]
pub enum Function {
    Native {
        #[unsafe_ignore_trace]
        function: NativeFunctionSignature,
        constructable: bool,
    },
    Closure {
        #[unsafe_ignore_trace]
        function: Box<dyn ClosureFunctionSignature>,
        constructable: bool,
        captures: Captures,
    },
    Ordinary {
        constructable: bool,
        this_mode: ThisMode,
        body: RcStatementList,
        params: Box<[FormalParameter]>,
        environment: Environment,
    },
    #[cfg(feature = "vm")]
    VmOrdinary {
        code: gc::Gc<crate::vm::CodeBlock>,
        environment: Environment,
    },
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Function {{ ... }}")
    }
}

impl Function {
    // Adds the final rest parameters to the Environment as an array
    #[cfg(not(feature = "vm"))]
    pub(crate) fn add_rest_param(
        param: &FormalParameter,
        index: usize,
        args_list: &[JsValue],
        context: &mut Context,
        local_env: &Environment,
    ) {
        
        use crate::builtins::Array;
        // Create array of values
        let array = Array::new_array(context);
        Array::add_to_array_object(&array, args_list.get(index..).unwrap_or_default(), context)
            .unwrap();


        let binding_params = param.run(Some(array.clone()), context).unwrap_or_default();
        for binding_items in binding_params.iter() {
            
            // // Create binding
            local_env
                .create_mutable_binding(binding_items.0.as_ref(), false, true, context)
                .expect("Failed to create binding");


            // // Set binding to value
            local_env
                .initialize_binding(binding_items.0.as_ref() , JsValue::new(binding_items.1.clone()), context)
                .expect("Failed to intialize binding");

        }
    }

    // Adds an argument to the environment
    pub(crate) fn add_arguments_to_environment(
        param: &FormalParameter,
        value: JsValue,
        local_env: &Environment,
        context: &mut Context,
    ) {


        let binding_params = param.run(Some(value.clone()), context).unwrap_or_default();
        for binding_items in binding_params.iter() {
           
            // // Create binding
            local_env
                .create_mutable_binding(binding_items.0.as_ref(), false, true, context)
                .expect("Failed to create binding");


            // // Set binding to value
            local_env
                .initialize_binding(binding_items.0.as_ref() , JsValue::new(binding_items.1.clone()), context)
                .expect("Failed to intialize binding");

        }

    }

    /// Returns true if the function object is constructable.
    pub fn is_constructable(&self) -> bool {
        match self {
            Self::Native { constructable, .. } => *constructable,
            Self::Closure { constructable, .. } => *constructable,
            Self::Ordinary { constructable, .. } => *constructable,
            #[cfg(feature = "vm")]
            Self::VmOrdinary { code, .. } => code.constructable,
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
    interpreter: &Context,
) where
    N: Into<String>,
{
    let name = name.into();
    let _timer = BoaProfiler::global().start_event(&format!("make_builtin_fn: {}", &name), "init");

    let function = JsObject::from_proto_and_data(
        interpreter.standard_objects().function_object().prototype(),
        ObjectData::function(Function::Native {
            function,
            constructable: false,
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

#[derive(Debug, Clone, Copy)]
pub struct BuiltInFunctionObject;

impl BuiltInFunctionObject {
    pub const LENGTH: usize = 1;

    fn constructor(
        new_target: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype =
            get_prototype_from_constructor(new_target, StandardObjects::function_object, context)?;

        let this = JsObject::from_proto_and_data(
            prototype,
            ObjectData::function(Function::Native {
                function: |_, _, _| Ok(JsValue::undefined()),
                constructable: true,
            }),
        );

        Ok(this.into())
    }

    fn prototype(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::undefined())
    }

    /// `Function.prototype.call`
    ///
    /// The call() method invokes self with the first argument as the `this` value.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-function.prototype.call
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function/call
    fn call(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if !this.is_function() {
            return context.throw_type_error(format!("{} is not a function", this.display()));
        }
        let this_arg = args.get_or_undefined(0);
        // TODO?: 3. Perform PrepareForTailCall
        let start = if !args.is_empty() { 1 } else { 0 };
        context.call(this, this_arg, &args[start..])
    }

    /// `Function.prototype.apply`
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
    fn apply(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        if !this.is_function() {
            return context.throw_type_error(format!("{} is not a function", this.display()));
        }
        let this_arg = args.get_or_undefined(0);
        let arg_array = args.get_or_undefined(1);
        if arg_array.is_null_or_undefined() {
            // TODO?: 3.a. PrepareForTailCall
            return context.call(this, this_arg, &[]);
        }
        let arg_list = arg_array.create_list_from_array_like(&[], context)?;
        // TODO?: 5. PrepareForTailCall
        context.call(this, this_arg, &arg_list)
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_string(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let name = {
            // Is there a case here where if there is no name field on a value
            // name should default to None? Do all functions have names set?
            let value = this.get_field("name", &mut *context)?;
            if value.is_null_or_undefined() {
                None
            } else {
                Some(value.to_string(context)?)
            }
        };

        let function = {
            let object = this
                .as_object()
                .map(|object| object.borrow().as_function().cloned());

            if let Some(Some(function)) = object {
                function
            } else {
                return context.throw_type_error("Not a function");
            }
        };

        match (&function, name) {
            (
                Function::Native {
                    function: _,
                    constructable: _,
                },
                Some(name),
            ) => Ok(format!("function {}() {{\n  [native Code]\n}}", &name).into()),
            (Function::Ordinary { body, params, .. }, Some(name)) => {
                
                let arguments: String = {
                    let mut argument_list: Vec<String> = Vec::new();
                    for params_item in params.iter(){
                        
                        argument_list.push (
                            match &params_item.declaration(){
                                Declaration::Identifier{ident, .. } => ident.as_ref().to_string(),
                                Declaration::Pattern(pattern) => {
                                    vec!["{ ".to_string() , pattern.idents().join(", ") , " }".to_string() ].join("")
                                    
                                    
                                  
                                }
                            }.clone()
                        );

                      
                    }
                    if argument_list.len() > 1 {
                        argument_list.join(", ")
                    }
                    else {
                        "".to_string()
                    }
                    // params
                    // .iter()
                    // .map(|param| param.name().iter().map(|param_name| param_name))
                    // .collect::<Vec<&str>>()
                    
                };

                let statement_list = &*body;
                // This is a kluge. The implementaion in browser seems to suggest that
                // the value here is printed exactly as defined in source. I'm not sure if
                // that's possible here, but for now here's a dumb heuristic that prints functions
                let is_multiline = {
                    let value = statement_list.to_string();
                    value.lines().count() > 1
                };
                if is_multiline {
                    Ok(
                        // ?? For some reason statement_list string implementation
                        // sticks a \n at the end no matter what
                        format!(
                            "{}({}) {{\n{}}}",
                            &name,
                            arguments,
                            statement_list.to_string()
                        )
                        .into(),
                    )
                } else {
                    Ok(format!(
                        "{}({}) {{{}}}",
                        &name,
                        arguments,
                        // The trim here is to remove a \n stuck at the end
                        // of the statement_list to_string method
                        statement_list.to_string().trim()
                    )
                    .into())
                }
            }

            _ => Ok("TODO".into()),
        }
    }
}

impl BuiltIn for BuiltInFunctionObject {
    const NAME: &'static str = "Function";

    const ATTRIBUTE: Attribute = Attribute::WRITABLE
        .union(Attribute::NON_ENUMERABLE)
        .union(Attribute::CONFIGURABLE);

    fn init(context: &mut Context) -> JsValue {
        let _timer = BoaProfiler::global().start_event("function", "init");

        let function_prototype = context.standard_objects().function_object().prototype();
        FunctionBuilder::native(context, Self::prototype)
            .name("")
            .length(0)
            .constructable(false)
            .build_function_prototype(&function_prototype);

        let function_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().function_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .method(Self::call, "call", 1)
        .method(Self::apply, "apply", 1)
        .method(Self::to_string, "toString", 0)
        .build();

        function_object.into()
    }
}
