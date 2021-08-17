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

use crate::object::PROTOTYPE;
use crate::{
    builtins::{Array, BuiltIn},
    environment::lexical_environment::Environment,
    gc::{custom_trace, empty_trace, Finalize, Trace},
    object::{ConstructorBuilder, FunctionBuilder, JsObject, Object, ObjectData},
    property::{Attribute, PropertyDescriptor},
    syntax::ast::node::{FormalParameter, RcStatementList},
    BoaProfiler, Context, JsResult, JsValue,
};
use bitflags::bitflags;
use std::fmt::{self, Debug};
use std::rc::Rc;

#[cfg(test)]
mod tests;

/// _fn(this, arguments, context) -> ResultValue_ - The signature of a native built-in function
pub type NativeFunction = fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue>;

/// _fn(this, arguments, context) -> ResultValue_ - The signature of a closure built-in function
pub type ClosureFunction = dyn Fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue>;

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
#[derive(Finalize)]
pub enum Function {
    Native {
        function: BuiltInFunction,
        constructable: bool,
    },
    Closure {
        function: Rc<ClosureFunction>,
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

unsafe impl Trace for Function {
    custom_trace!(this, {
        match this {
            Function::Native { .. } => {}
            Function::Closure { .. } => {}
            Function::Ordinary { environment, .. } => {
                mark(environment);
            }
        }
    });
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
            .create_mutable_binding(param.name().to_owned(), false, true, context)
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
            .create_mutable_binding(param.name().to_owned(), false, true, context)
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
        let prototype = new_target
            .as_object()
            .and_then(|obj| {
                obj.__get__(&PROTOTYPE.into(), obj.clone().into(), context)
                    .map(|o| o.as_object())
                    .transpose()
            })
            .transpose()?
            .unwrap_or_else(|| context.standard_objects().object_object().prototype());
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
        let this_arg: JsValue = args.get(0).cloned().unwrap_or_default();
        // TODO?: 3. Perform PrepareForTailCall
        let start = if !args.is_empty() { 1 } else { 0 };
        context.call(this, &this_arg, &args[start..])
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
        let this_arg = args.get(0).cloned().unwrap_or_default();
        let arg_array = args.get(1).cloned().unwrap_or_default();
        if arg_array.is_null_or_undefined() {
            // TODO?: 3.a. PrepareForTailCall
            return context.call(this, &this_arg, &[]);
        }
        let arg_array = arg_array.as_object().ok_or_else(|| {
            context.construct_type_error("argList must be null, undefined or an object")
        })?;
        let arg_list = arg_array.create_list_from_array_like(&[], context)?;
        // TODO?: 5. PrepareForTailCall
        context.call(this, &this_arg, &arg_list)
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
        .build();

        (Self::NAME, function_object.into(), Self::attribute())
    }
}
