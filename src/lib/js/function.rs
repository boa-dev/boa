use gc::{Gc, GcCell};
use js::object::{ObjectData, Property};
use js::value::{to_value, ResultValue, Value, ValueData};
use std::collections::HashMap;
use std::iter::FromIterator;

pub type FunctionData = fn(Vec<Value>, Value, Value, Value) -> ResultValue;
/// A Javascript function
/// A member of the Object type that may be invoked as a subroutine
/// https://tc39.github.io/ecma262/#sec-terms-and-definitions-function
/// In our implementation, Function is extending Object by holding an object field which some extra data

#[derive(Trace, Finalize, Debug)]
pub struct Function {
    /// The fields associated with the function
    pub object: ObjectData,
    /// This function's JIT representation
    pub repr: FunctionData,
    /// The argument names of the function
    pub args: Vec<String>,
}

impl Function {
    /// Make a new function
    pub fn new(repr: FunctionData, args: Vec<String>) -> Function {
        let mut obj = HashMap::new();
        obj.insert(
            "arguments".to_string(),
            Property::from_value(to_value(args.len() as i32)),
        );
        Function {
            object: obj,
            repr: repr,
            args: args,
        }
    }
    /// Create a function from function data and arguments
    pub fn make(repr: FunctionData, args: &[&'static str]) -> Value {
        Value {
            ptr: Gc::new(ValueData::Function(GcCell::new(Function::new(
                repr,
                FromIterator::from_iter(args.iter().map(|arg| arg.to_string())),
            )))),
        }
    }
    /// Call with some args
    pub fn call(&self, args: Vec<Value>, global: Value, scope: Value, this: Value) -> ResultValue {
        (self.repr)(args, global, scope, this)
    }
}

/// Create a new `Function` object
pub fn _create() -> Value {
    let function: ObjectData = HashMap::new();
    to_value(function)
}
/// Initialise the global object with the `Function` object
pub fn init(global: Value) {
    global.set_field_slice("Function", _create());
}
