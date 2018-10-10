use exec::Interpreter;
use gc::GcCell;
use js::object::{ObjectData, Property};
use js::value::{to_value, ResultValue, Value, ValueData};
use std::collections::HashMap;
use syntax::ast::expr::Expr;
/// A Javascript function
/// A member of the Object type that may be invoked as a subroutine
/// https://tc39.github.io/ecma262/#sec-terms-and-definitions-function
pub enum Function {
    /// A native javascript function
    NativeFunc(NativeFunction),
    /// A regular javascript function
    RegularFunc(RegularFunction),
}

impl Function {
    /// Call a function with some arguments
    pub fn call(
        &self,
        exe: &mut Interpreter,
        this: Value,
        callee: Value,
        args: Vec<Value>,
    ) -> ResultValue {
        match *self {
            Function::NativeFunc(ref ntv) => {
                let func = ntv.data;
                func(this, callee, args)
            }
            Function::RegularFunc(ref data) => {
                let scope = exe.make_scope();
                scope
                    .borrow()
                    .borrow_mut()
                    .insert("this".to_string(), Property::new(this));
                for i in 0..data.args.len() {
                    let name = data.args.get(i);
                    let expr = args.get(i);
                    scope
                        .borrow()
                        .borrow_mut()
                        .insert(name.to_string(), Property::new(*expr));
                }
                let result = exe.run(&data.expr);
                exe.destroy_scope();
                result
            }
        }
    }
}

/// Represents a regular javascript function in memory
/// A member of the Object type that may be invoked as a subroutine
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
    pub fn new(expr: Expr, args: Vec<String>) -> RegularFunction {
        let mut obj = HashMap::new();
        obj.insert(
            "arguments".to_string(),
            Property::new(GcCell::new(ValueData::Integer(args.len() as i32))),
        );
        RegularFunction {
            object: obj,
            expr: expr,
            args: args,
        }
    }
}

pub type NativeFunctionData = fn(Value, Value, Vec<Value>) -> ResultValue;

/// Represents a native javascript function in memory
pub struct NativeFunction {
    /// The fields associated with the function
    pub object: ObjectData,
    /// The callable function data
    pub data: NativeFunctionData,
}
impl NativeFunction {
    /// Make a new native function with the given function data
    pub fn new(data: NativeFunctionData) -> NativeFunction {
        let obj = HashMap::new();
        NativeFunction {
            object: obj,
            data: data,
        }
    }
}

/// Create a new `Function` object
pub fn _create() -> Value {
    let function: ObjectData = HashMap::new();
    to_value(function)
}

/// Initialise the global object with the `Function` object
pub fn init(global: Value) {
    let global_ptr = global.borrow();
    global_ptr.set_field_slice("Function", _create(global));
}
