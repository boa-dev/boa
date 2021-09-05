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

use crate::context::StandardObjects;
use crate::object::internal_methods::get_prototype_from_constructor;

use crate::{
    builtins::{Array, BuiltIn},
    environment::lexical_environment::Environment,
    gc::{empty_trace, Finalize, Trace},
    object::{ConstructorBuilder, FunctionBuilder, JsObject, Object, ObjectData},
    property::{Attribute, PropertyDescriptor},
    syntax::ast::node::{FormalParameter, RcStatementList},
    BoaProfiler, Context, JsResult, JsValue,
};
use bitflags::bitflags;
use dyn_clone::DynClone;
use sealed::Sealed;
use std::fmt::{self, Debug};

use super::JsArgs;

#[cfg(test)]
mod tests;

// Allows restricting closures to only `Copy` ones.
// Used the sealed pattern to disallow external implementations
// of `DynCopy`.
mod sealed {
    pub trait Sealed {}
    impl<T: Copy> Sealed for T {}
}
pub trait DynCopy: Sealed {}
impl<T: Copy> DynCopy for T {}

/// Type representing a native built-in function a.k.a. function pointer.
///
/// Native functions need to have this signature in order to
/// be callable from Javascript.
pub type NativeFunction = fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue>;

/// Trait representing a native built-in closure.
///
/// Closures need to have this signature in order to
/// be callable from Javascript, but most of the time the compiler
/// is smart enough to correctly infer the types.
pub trait ClosureFunction:
    Fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue> + DynCopy + DynClone + 'static
{
}

// The `Copy` bound automatically infers `DynCopy` and `DynClone`
impl<T> ClosureFunction for T where
    T: Fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue> + Copy + 'static
{
}

// Allows cloning Box<dyn ClosureFunction>
dyn_clone::clone_trait_object!(ClosureFunction);

#[derive(Clone, Copy, Finalize)]
pub struct BuiltInFunction(pub(crate) NativeFunction);

unsafe impl Trace for BuiltInFunction {
    empty_trace!();
}

impl From<NativeFunction> for BuiltInFunction {
    fn from(function: NativeFunction) -> Self {
        Self(function)
    }
}

impl Debug for BuiltInFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[native]")
    }
}

bitflags! {
    #[derive(Finalize, Default)]
    pub struct FunctionFlags: u8 {
        const CONSTRUCTABLE = 0b0000_0010;
        const LEXICAL_THIS_MODE = 0b0000_0100;
    }
}

impl FunctionFlags {
    #[inline]
    pub(crate) fn is_constructable(&self) -> bool {
        self.contains(Self::CONSTRUCTABLE)
    }

    #[inline]
    pub(crate) fn is_lexical_this_mode(&self) -> bool {
        self.contains(Self::LEXICAL_THIS_MODE)
    }
}

unsafe impl Trace for FunctionFlags {
    empty_trace!();
}

/// Boa representation of a Function Object.
///
/// FunctionBody is specific to this interpreter, it will either be Rust code or JavaScript code (AST Node)
///
/// <https://tc39.es/ecma262/#sec-ecmascript-function-objects>
#[derive(Clone, Trace, Finalize)]
pub enum Function {
    Native {
        function: BuiltInFunction,
        constructable: bool,
    },
    Closure {
        #[unsafe_ignore_trace]
        function: Box<dyn ClosureFunction>,
        constructable: bool,
    },
    Ordinary {
        flags: FunctionFlags,
        body: RcStatementList,
        params: Box<[FormalParameter]>,
        environment: Environment,
    },
}

impl Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Function {{ ... }}")
    }
}

impl Function {
    // Adds the final rest parameters to the Environment as an array
    pub(crate) fn add_rest_param(
        &self,
        param: &FormalParameter,
        index: usize,
        args_list: &[JsValue],
        context: &mut Context,
        local_env: &Environment,
    ) {
        // Create array of values
        let array = Array::new_array(context);
        Array::add_to_array_object(&array, args_list.get(index..).unwrap_or_default(), context)
            .unwrap();

        // Create binding
        local_env
            // Function parameters can share names in JavaScript...
            .create_mutable_binding(param.name(), false, true, context)
            .expect("Failed to create binding for rest param");

        // Set Binding to value
        local_env
            .initialize_binding(param.name(), array, context)
            .expect("Failed to initialize rest param");
    }

    // Adds an argument to the environment
    pub(crate) fn add_arguments_to_environment(
        &self,
        param: &FormalParameter,
        value: JsValue,
        local_env: &Environment,
        context: &mut Context,
    ) {
        // Create binding
        local_env
            .create_mutable_binding(param.name(), false, true, context)
            .expect("Failed to create binding");

        // Set Binding to value
        local_env
            .initialize_binding(param.name(), value, context)
            .expect("Failed to intialize binding");
    }

    /// Returns true if the function object is constructable.
    pub fn is_constructable(&self) -> bool {
        match self {
            Self::Native { constructable, .. } => *constructable,
            Self::Closure { constructable, .. } => *constructable,
            Self::Ordinary { flags, .. } => flags.is_constructable(),
        }
    }
}

/// Arguments.
///
/// <https://tc39.es/ecma262/#sec-createunmappedargumentsobject>
pub fn create_unmapped_arguments_object(
    arguments_list: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let len = arguments_list.len();
    let obj = JsObject::new(Object::default());
    // Set length
    let length = PropertyDescriptor::builder()
        .value(len)
        .writable(true)
        .enumerable(false)
        .configurable(true);
    // Define length as a property
    crate::object::internal_methods::ordinary_define_own_property(
        &obj,
        "length".into(),
        length.into(),
        context,
    )?;
    let mut index: usize = 0;
    while index < len {
        let val = arguments_list.get(index).expect("Could not get argument");
        let prop = PropertyDescriptor::builder()
            .value(val.clone())
            .writable(true)
            .enumerable(true)
            .configurable(true);

        obj.insert(index, prop);
        index += 1;
    }

    Ok(JsValue::new(obj))
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
pub fn make_builtin_fn<N>(
    function: NativeFunction,
    name: N,
    parent: &JsObject,
    length: usize,
    interpreter: &Context,
) where
    N: Into<String>,
{
    let name = name.into();
    let _timer = BoaProfiler::global().start_event(&format!("make_builtin_fn: {}", &name), "init");

    let mut function = Object::function(
        Function::Native {
            function: function.into(),
            constructable: false,
        },
        interpreter
            .standard_objects()
            .function_object()
            .prototype()
            .into(),
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
        let this = JsValue::new_object(context);

        this.as_object()
            .expect("this should be an object")
            .set_prototype_instance(prototype.into());

        this.set_data(ObjectData::function(Function::Native {
            function: BuiltInFunction(|_, _, _| Ok(JsValue::undefined())),
            constructable: true,
        }));
        Ok(this)
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
                let arguments: String = params
                    .iter()
                    .map(|param| param.name())
                    .collect::<Vec<&str>>()
                    .join(", ");

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

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, JsValue, Attribute) {
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

        (Self::NAME, function_object.into(), Self::attribute())
    }
}
