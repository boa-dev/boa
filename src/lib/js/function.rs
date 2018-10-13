use js::object::ObjectData;
use js::value::{ResultValue, Value};

pub type FunctionData = fn(Vec<Value>, Value, Value, Value) -> ResultValue;
/// A Javascript function
/// A member of the Object type that may be invoked as a subroutine
/// https://tc39.github.io/ecma262/#sec-terms-and-definitions-function
/// In our implementation, Function is extending Object by holding an object field which some extra data

#[derive(Trace, Finalize)]
pub struct Function {
    /// The fields associated with the function
    pub object: ObjectData,
    /// This function's JIT representation
    pub repr: FunctionData,
    /// The argument names of the function
    pub args: Vec<String>,
}
