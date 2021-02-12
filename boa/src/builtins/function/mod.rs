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
    gc::{empty_trace, Finalize, Trace},
    object::{ConstructorBuilder, FunctionBuilder, GcObject, Object, ObjectData},
    property::{Attribute, DataDescriptor},
    syntax::ast::node::{FormalParameter, RcStatementList},
    BoaProfiler, Context, Result, Value,
};
use bitflags::bitflags;
use std::fmt::{self, Debug};

#[cfg(test)]
mod tests;

/// _fn(this, arguments, context) -> ResultValue_ - The signature of a built-in function
pub type NativeFunction = fn(&Value, &[Value], &mut Context) -> Result<Value>;

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
        const CALLABLE = 0b0000_0001;
        const CONSTRUCTABLE = 0b0000_0010;
        const LEXICAL_THIS_MODE = 0b0000_0100;
    }
}

impl FunctionFlags {
    pub(crate) fn from_parameters(callable: bool, constructable: bool) -> Self {
        let mut flags = Self::default();

        if callable {
            flags |= Self::CALLABLE;
        }
        if constructable {
            flags |= Self::CONSTRUCTABLE;
        }

        flags
    }

    #[inline]
    pub(crate) fn is_callable(&self) -> bool {
        self.contains(Self::CALLABLE)
    }

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
#[derive(Debug, Clone, Finalize, Trace)]
pub enum Function {
    BuiltIn(BuiltInFunction, FunctionFlags),
    Ordinary {
        flags: FunctionFlags,
        body: RcStatementList,
        params: Box<[FormalParameter]>,
        environment: Environment,
    },
}

impl Function {
    // Adds the final rest parameters to the Environment as an array
    pub(crate) fn add_rest_param(
        &self,
        param: &FormalParameter,
        index: usize,
        args_list: &[Value],
        context: &mut Context,
        local_env: &Environment,
    ) {
        // Create array of values
        let array = Array::new_array(context);
        Array::add_to_array_object(&array, &args_list[index..], context).unwrap();

        // Create binding
        local_env
            .borrow_mut()
            // Function parameters can share names in JavaScript...
            .create_mutable_binding(param.name().to_owned(), false, true)
            .expect("Failed to create binding for rest param");

        // Set Binding to value
        local_env
            .borrow_mut()
            .initialize_binding(param.name(), array)
            .expect("Failed to initialize rest param");
    }

    // Adds an argument to the environment
    pub(crate) fn add_arguments_to_environment(
        &self,
        param: &FormalParameter,
        value: Value,
        local_env: &Environment,
    ) {
        // Create binding
        local_env
            .borrow_mut()
            .create_mutable_binding(param.name().to_owned(), false, true)
            .expect("Failed to create binding");

        // Set Binding to value
        local_env
            .borrow_mut()
            .initialize_binding(param.name(), value)
            .expect("Failed to intialize binding");
    }

    /// Returns true if the function object is callable.
    pub fn is_callable(&self) -> bool {
        match self {
            Self::BuiltIn(_, flags) => flags.is_callable(),
            Self::Ordinary { flags, .. } => flags.is_callable(),
        }
    }

    /// Returns true if the function object is constructable.
    pub fn is_constructable(&self) -> bool {
        match self {
            Self::BuiltIn(_, flags) => flags.is_constructable(),
            Self::Ordinary { flags, .. } => flags.is_constructable(),
        }
    }
}

/// Arguments.
///
/// <https://tc39.es/ecma262/#sec-createunmappedargumentsobject>
pub fn create_unmapped_arguments_object(arguments_list: &[Value]) -> Value {
    let len = arguments_list.len();
    let mut obj = GcObject::new(Object::default());
    // Set length
    let length = DataDescriptor::new(
        len,
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
    );
    // Define length as a property
    obj.ordinary_define_own_property("length", length.into());
    let mut index: usize = 0;
    while index < len {
        let val = arguments_list.get(index).expect("Could not get argument");
        let prop = DataDescriptor::new(
            val.clone(),
            Attribute::WRITABLE | Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
        );

        obj.insert(index, prop);
        index += 1;
    }

    Value::from(obj)
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
    parent: &GcObject,
    length: usize,
    interpreter: &Context,
) where
    N: Into<String>,
{
    let name = name.into();
    let _timer = BoaProfiler::global().start_event(&format!("make_builtin_fn: {}", &name), "init");

    let mut function = Object::function(
        Function::BuiltIn(function.into(), FunctionFlags::CALLABLE),
        interpreter
            .standard_objects()
            .function_object()
            .prototype()
            .into(),
    );
    let attribute = Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE;
    function.insert_property("length", length, attribute);
    function.insert_property("name", name.as_str(), attribute);

    parent.clone().insert_property(
        name,
        function,
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
    );
}

#[derive(Debug, Clone, Copy)]
pub struct BuiltInFunctionObject;

impl BuiltInFunctionObject {
    pub const LENGTH: usize = 1;

    fn constructor(new_target: &Value, _: &[Value], context: &mut Context) -> Result<Value> {
        let prototype = new_target
            .as_object()
            .and_then(|obj| {
                obj.get(&PROTOTYPE.into(), obj.clone().into(), context)
                    .map(|o| o.as_object())
                    .transpose()
            })
            .transpose()?
            .unwrap_or_else(|| context.standard_objects().object_object().prototype());
        let this = Value::new_object(context);

        this.as_object()
            .expect("this should be an object")
            .set_prototype_instance(prototype.into());

        this.set_data(ObjectData::Function(Function::BuiltIn(
            BuiltInFunction(|_, _, _| Ok(Value::undefined())),
            FunctionFlags::CALLABLE | FunctionFlags::CONSTRUCTABLE,
        )));
        Ok(this)
    }

    #[allow(clippy::unnecessary_wraps)] // built-in function
    fn prototype(_: &Value, _: &[Value], _: &mut Context) -> Result<Value> {
        Ok(Value::undefined())
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
    fn call(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        if !this.is_function() {
            return context.throw_type_error(format!("{} is not a function", this.display()));
        }
        let this_arg: Value = args.get(0).cloned().unwrap_or_default();
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
    fn apply(this: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        if !this.is_function() {
            return context.throw_type_error(format!("{} is not a function", this.display()));
        }
        let this_arg = args.get(0).cloned().unwrap_or_default();
        let arg_array = args.get(1).cloned().unwrap_or_default();
        if arg_array.is_null_or_undefined() {
            // TODO?: 3.a. PrepareForTailCall
            return context.call(this, &this_arg, &[]);
        }
        let arg_list = context
            .extract_array_properties(&arg_array)?
            .map_err(|()| arg_array)?;
        // TODO?: 5. PrepareForTailCall
        context.call(this, &this_arg, &arg_list)
    }
}

impl BuiltIn for BuiltInFunctionObject {
    const NAME: &'static str = "Function";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event("function", "init");

        let function_prototype = context.standard_objects().function_object().prototype();
        FunctionBuilder::new(context, Self::prototype)
            .name("")
            .length(0)
            .callable(true)
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
