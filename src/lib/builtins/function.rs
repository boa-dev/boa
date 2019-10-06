use crate::{
    exec::Interpreter,
    builtins::{
        object::Object,
        property::Property,
        value::{to_value, ResultValue, Value, ValueData},
    },
    syntax::ast::expr::Expr,
};
use gc::{custom_trace, Gc};
use gc_derive::{Finalize, Trace};
use std::fmt::{self, Debug};

/// fn(this, arguments, ctx)
pub type NativeFunctionData = fn(&Value, &[Value], &mut Interpreter) -> ResultValue;

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
    pub object: Object,
    /// This function's expression
    pub expr: Expr,
    /// The argument names of the function
    pub args: Vec<String>,
}

impl RegularFunction {
    /// Make a new regular function
    #[allow(clippy::cast_possible_wrap)]
    pub fn new(expr: Expr, args: Vec<String>) -> Self {
        let mut object = Object::default();
        object.properties.insert(
            "arguments".to_string(),
            Property::default().value(Gc::new(ValueData::Integer(args.len() as i32))),
        );
        Self { object, expr, args }
    }
}

#[derive(Finalize, Clone)]
/// Represents a native javascript function in memory
pub struct NativeFunction {
    /// The fields associated with the function
    pub object: Object,
    /// The callable function data
    pub data: NativeFunctionData,
}

impl NativeFunction {
    /// Make a new native function with the given function data
    pub fn new(data: NativeFunctionData) -> Self {
        let object = Object::default();
        Self { object, data }
    }
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for (key, val) in self.object.properties.iter() {
            write!(
                f,
                "{}: {}",
                key,
                val.value
                    .as_ref()
                    .unwrap_or(&Gc::new(ValueData::Undefined))
                    .clone()
            )?;
        }
        write!(f, "}}")
    }
}

unsafe impl gc::Trace for NativeFunction {
    custom_trace!(this, mark(&this.object));
}

/// Create a new `Function` object
pub fn _create() -> Value {
    let function: Object = Object::default();
    to_value(function)
}
/// Initialise the global object with the `Function` object
pub fn init(global: &Value) {
    let global_ptr = global;
    global_ptr.set_field_slice("Function", _create());
}

/// Arguments
/// https://tc39.es/ecma262/#sec-createunmappedargumentsobject
pub fn create_unmapped_arguments_object(arguments_list: Vec<Value>) -> Value {
    let len = arguments_list.len();
    let mut obj = Object::default();
    obj.set_internal_slot("ParameterMap", Gc::new(ValueData::Undefined));
    // Set length
    let mut length = Property::default();
    length = length.writable(true).value(to_value(len));
    // Define length as a property
    obj.define_own_property("length".to_string(), length);
    let mut index: usize = 0;
    while index < len {
        let val = arguments_list.get(index).unwrap();
        let mut prop = Property::default();
        prop = prop
            .value(val.clone())
            .enumerable(true)
            .writable(true)
            .configurable(true);

        obj.properties.insert(index.to_string(), prop);
        index += 1;
    }

    to_value(obj)
}

#[cfg(test)]
mod tests {
    use crate::exec::Executor;
<<<<<<< HEAD:src/lib/builtins/function.rs
    use crate::{forward, forward_val, builtins::value::from_value};
=======
    use crate::realm::Realm;
    use crate::{forward, forward_val, js::value::from_value};
>>>>>>> upstream/master:src/lib/js/function.rs

    #[test]
    fn check_arguments_object() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        function jason(a, b) {
            return arguments[0];
        }
        const val = jason(100, 6);
        "#;

        forward(&mut engine, init);
        let return_val = forward_val(&mut engine, "val").expect("value expected");
        assert_eq!(return_val.is_double(), true);
        assert_eq!(from_value::<f64>(return_val).unwrap(), 100.0);
    }
}
