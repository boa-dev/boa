use crate::{
    exec::Interpreter,
    js::{
        object::{ObjectData, Property},
        value::{to_value, ResultValue, Value, ValueData},
    },
    syntax::ast::expr::Expr,
};
use gc::{custom_trace, Gc};
use gc_derive::{Finalize, Trace};
use std::fmt::{self, Debug};

/// fn(this, arguments, ctx)
pub type NativeFunctionData = fn(&Value, &[Value], &Interpreter) -> ResultValue;

/// A Javascript function
/// A member of the Object type that may be invoked as a subroutine
/// <https://tc39.github.io/ecma262/#sec-terms-and-definitions-function>
/// In our implementation, Function is extending Object by holding an object field which some extra data

/// A Javascript function
#[derive(Trace, Finalize, Debug, Clone)]
pub enum Function {
    /// A native javascript function
    NativeFunc(NativeFunction),
    /// A regular javascript function
    RegularFunc(RegularFunction),
}

/// Represents a regular javascript function in memory
#[derive(Trace, Finalize, Debug, Clone)]
pub struct RegularFunction {
    /// The fields associated with the function
    pub object: ObjectData,
    /// This function's expression
    pub expr: Expr,
    /// The argument names of the function
    pub args: Vec<String>,
}

impl RegularFunction {
    /// Make a new regular function
    #[allow(clippy::cast_possible_wrap)]
    pub fn new(expr: Expr, args: Vec<String>) -> Self {
        let mut object = ObjectData::default();
        object.properties.insert(
            "arguments".to_string(),
            Property::new(Gc::new(ValueData::Integer(args.len() as i32))),
        );
        Self { object, expr, args }
    }
}

#[derive(Finalize, Clone)]
/// Represents a native javascript function in memory
pub struct NativeFunction {
    /// The fields associated with the function
    pub object: ObjectData,
    /// The callable function data
    pub data: NativeFunctionData,
}

impl NativeFunction {
    /// Make a new native function with the given function data
    pub fn new(data: NativeFunctionData) -> Self {
        let object = ObjectData::default();
        Self { object, data }
    }
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for (key, val) in self.object.properties.iter() {
            write!(f, "{}: {}", key, val.value.clone())?;
        }
        write!(f, "}}")
    }
}

unsafe impl gc::Trace for NativeFunction {
    custom_trace!(this, mark(&this.object));
}

/// Create a new `Function` object
pub fn _create() -> Value {
    let function: ObjectData = ObjectData::default();
    to_value(function)
}
/// Initialise the global object with the `Function` object
pub fn init(global: &Value) {
    let global_ptr = global;
    global_ptr.set_field_slice("Function", _create());
}
