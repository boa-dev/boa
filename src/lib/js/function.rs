use crate::js::object::{ObjectData, Property};
use crate::js::value::{to_value, ResultValue, Value, ValueData};
use crate::syntax::ast::expr::Expr;
use gc::Gc;
use std::collections::HashMap;

/// fn(this, callee, arguments)
pub type NativeFunctionData = fn(Value, Value, Vec<Value>) -> ResultValue;

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
    pub fn new(expr: Expr, args: Vec<String>) -> Self {
        let mut object = HashMap::new();
        object.insert(
            "arguments".to_string(),
            Property::new(Gc::new(ValueData::Integer(args.len() as i32))),
        );
        Self { object, expr, args }
    }
}

#[derive(Trace, Finalize, Debug, Clone)]
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
        let object = HashMap::new();
        Self { object, data }
    }
}

/// Create a new `Function` object
pub fn _create() -> Value {
    let function: ObjectData = HashMap::new();
    to_value(function)
}
/// Initialise the global object with the `Function` object
pub fn init(global: &Value) {
    let global_ptr = global;
    global_ptr.set_field_slice("Function", _create());
}
