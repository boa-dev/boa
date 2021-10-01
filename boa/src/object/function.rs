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

use crate::{
    environment::lexical_environment::Environment,
    gc::{Finalize, Trace},
    object::{JsObject, Object},
    property::PropertyDescriptor,
    syntax::ast::node::{FormalParameter, RcStatementList},
    BoaProfiler, Context, JsResult, JsValue,
};

use dyn_clone::DynClone;
use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use super::NativeObject;

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
            Self::Ordinary { constructable, .. } => *constructable,
            #[cfg(feature = "vm")]
            Self::VmOrdinary { code, .. } => code.constructable,
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
        .configurable(true)
        .build();
    // Define length as a property
    crate::object::internal_methods::ordinary_define_own_property(
        &obj,
        "length".into(),
        length,
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

    let mut function = Object::function(
        Function::Native {
            function,
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
