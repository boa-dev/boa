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
    builtins::{
        array::Array,
        object::{Object, ObjectInternalMethods, ObjectKind, INSTANCE_PROTOTYPE, PROTOTYPE},
        property::Property,
        value::{ResultValue, Value},
    },
    environment::function_environment_record::BindingStatus,
    environment::lexical_environment::{new_function_environment, Environment},
    exec::{Executable, Interpreter},
    syntax::ast::node::{FormalParameter, StatementList},
    BoaProfiler,
};
use gc::{unsafe_empty_trace, Finalize, Trace};
use std::fmt::{self, Debug};

/// _fn(this, arguments, ctx) -> ResultValue_ - The signature of a built-in function
pub type NativeFunctionData = fn(&mut Value, &[Value], &mut Interpreter) -> ResultValue;

/// Sets the ConstructorKind
#[derive(Debug, Copy, Clone)]
pub enum ConstructorKind {
    Base,
    Derived,
}

/// Defines how this references are interpreted within the formal parameters and code body of the function.
///
/// Arrow functions don't define a `this` and thus are lexical, `function`s do define a this and thus are NonLexical
#[derive(Trace, Finalize, Debug, Clone)]
pub enum ThisMode {
    Lexical,
    NonLexical,
}

/// FunctionBody is specific to this interpreter, it will either be Rust code or JavaScript code (AST Node)
#[derive(Clone, Finalize)]
pub enum FunctionBody {
    BuiltIn(NativeFunctionData),
    Ordinary(StatementList),
}

impl Debug for FunctionBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuiltIn(_) => write!(f, "native code"),
            Self::Ordinary(statements) => write!(f, "{:?}", statements),
        }
    }
}

/// `Trace` implementation for `FunctionBody`.
///
/// This is indeed safe, but we need to mark this as an empty trace because neither
// `NativeFunctionData` nor Node hold any GC'd objects, but Gc doesn't know that. So we need to
/// signal it manually. `rust-gc` does not have a `Trace` implementation for `fn(_, _, _)`.
///
/// <https://github.com/Manishearth/rust-gc/blob/master/gc/src/trace.rs>
unsafe impl Trace for FunctionBody {
    unsafe_empty_trace!();
}

/// Boa representation of a Function Object.
///
/// <https://tc39.es/ecma262/#sec-ecmascript-function-objects>
#[derive(Trace, Finalize, Clone)]
pub struct Function {
    /// Call/Construct Function body
    pub body: FunctionBody,
    /// Formal Paramaters
    pub params: Box<[FormalParameter]>,
    /// This Mode
    pub this_mode: ThisMode,
    // Environment, built-in functions don't need Environments
    pub environment: Option<Environment>,
    /// Is it constructable
    constructable: bool,
    /// Is it callable.
    callable: bool,
}

impl Function {
    pub fn new<P>(
        parameter_list: P,
        scope: Option<Environment>,
        body: FunctionBody,
        this_mode: ThisMode,
        constructable: bool,
        callable: bool,
    ) -> Self
    where
        P: Into<Box<[FormalParameter]>>,
    {
        Self {
            body,
            environment: scope,
            params: parameter_list.into(),
            this_mode,
            constructable,
            callable,
        }
    }

    /// This will create an ordinary function object
    ///
    /// <https://tc39.es/ecma262/#sec-ordinaryfunctioncreate>
    pub fn ordinary<P>(
        parameter_list: P,
        scope: Environment,
        body: StatementList,
        this_mode: ThisMode,
    ) -> Self
    where
        P: Into<Box<[FormalParameter]>>,
    {
        Self::new(
            parameter_list.into(),
            Some(scope),
            FunctionBody::Ordinary(body),
            this_mode,
            true,
            true,
        )
    }

    /// This will create a built-in function object
    ///
    /// <https://tc39.es/ecma262/#sec-createbuiltinfunction>
    pub fn builtin<P>(parameter_list: P, body: NativeFunctionData) -> Self
    where
        P: Into<Box<[FormalParameter]>>,
    {
        let _timer = BoaProfiler::global().start_event("function::builtin", "function");
        Self::new(
            parameter_list.into(),
            None,
            FunctionBody::BuiltIn(body),
            ThisMode::NonLexical,
            false,
            true,
        )
    }

    /// This will handle calls for both ordinary and built-in functions
    ///
    /// <https://tc39.es/ecma262/#sec-prepareforordinarycall>
    /// <https://tc39.es/ecma262/#sec-ecmascript-function-objects-call-thisargument-argumentslist>
    pub fn call(
        &self,
        this: &mut Value, // represents a pointer to this function object wrapped in a GC (not a `this` JS object)
        args_list: &[Value],
        interpreter: &mut Interpreter,
        this_obj: &mut Value,
    ) -> ResultValue {
        let _timer = BoaProfiler::global().start_event("function::call", "function");
        if self.callable {
            match self.body {
                FunctionBody::BuiltIn(func) => func(this_obj, args_list, interpreter),
                FunctionBody::Ordinary(ref body) => {
                    // Create a new Function environment who's parent is set to the scope of the function declaration (self.environment)
                    // <https://tc39.es/ecma262/#sec-prepareforordinarycall>
                    let local_env = new_function_environment(
                        this.clone(),
                        None,
                        Some(self.environment.as_ref().unwrap().clone()),
                        BindingStatus::Uninitialized,
                    );

                    // Add argument bindings to the function environment
                    for i in 0..self.params.len() {
                        let param = self.params.get(i).expect("Could not get param");
                        // Rest Parameters
                        if param.is_rest_param() {
                            self.add_rest_param(param, i, args_list, interpreter, &local_env);
                            break;
                        }

                        let value = args_list.get(i).expect("Could not get value");
                        self.add_arguments_to_environment(param, value.clone(), &local_env);
                    }

                    // Add arguments object
                    let arguments_obj = create_unmapped_arguments_object(args_list);
                    local_env
                        .borrow_mut()
                        .create_mutable_binding("arguments".to_string(), false);
                    local_env
                        .borrow_mut()
                        .initialize_binding("arguments", arguments_obj);

                    interpreter.realm.environment.push(local_env);

                    // Call body should be set before reaching here
                    let result = body.run(interpreter);

                    // local_env gets dropped here, its no longer needed
                    interpreter.realm.environment.pop();
                    result
                }
            }
        } else {
            panic!("TypeError: class constructors must be invoked with 'new'");
        }
    }

    /// <https://tc39.es/ecma262/#sec-ecmascript-function-objects-construct-argumentslist-newtarget>
    pub fn construct(
        &self,
        this: &mut Value, // represents a pointer to this function object wrapped in a GC (not a `this` JS object)
        args_list: &[Value],
        interpreter: &mut Interpreter,
        this_obj: &mut Value,
    ) -> ResultValue {
        if self.constructable {
            match self.body {
                FunctionBody::BuiltIn(func) => {
                    func(this_obj, args_list, interpreter)?;
                    Ok(this_obj.clone())
                }
                FunctionBody::Ordinary(ref body) => {
                    // Create a new Function environment who's parent is set to the scope of the function declaration (self.environment)
                    // <https://tc39.es/ecma262/#sec-prepareforordinarycall>
                    let local_env = new_function_environment(
                        this.clone(),
                        Some(this_obj.clone()),
                        Some(self.environment.as_ref().unwrap().clone()),
                        BindingStatus::Initialized,
                    );

                    // Add argument bindings to the function environment
                    for (i, param) in self.params.iter().enumerate() {
                        // Rest Parameters
                        if param.is_rest_param() {
                            self.add_rest_param(param, i, args_list, interpreter, &local_env);
                            break;
                        }

                        let value = args_list.get(i).expect("Could not get value");
                        self.add_arguments_to_environment(param, value.clone(), &local_env);
                    }

                    // Add arguments object
                    let arguments_obj = create_unmapped_arguments_object(args_list);
                    local_env
                        .borrow_mut()
                        .create_mutable_binding("arguments".to_string(), false);
                    local_env
                        .borrow_mut()
                        .initialize_binding("arguments", arguments_obj);

                    interpreter.realm.environment.push(local_env);

                    // Call body should be set before reaching here
                    let _ = body.run(interpreter);

                    // local_env gets dropped here, its no longer needed
                    let binding = interpreter.realm.environment.get_this_binding();
                    Ok(binding)
                }
            }
        } else {
            let name = this.get_field("name").to_string();
            panic!("TypeError: {} is not a constructor", name);
        }
    }

    // Adds the final rest parameters to the Environment as an array
    fn add_rest_param(
        &self,
        param: &FormalParameter,
        index: usize,
        args_list: &[Value],
        interpreter: &mut Interpreter,
        local_env: &Environment,
    ) {
        // Create array of values
        let array = Array::new_array(interpreter).unwrap();
        Array::add_to_array_object(&array, &args_list[index..]).unwrap();

        // Create binding
        local_env
            .borrow_mut()
            .create_mutable_binding(param.name().to_owned(), false);

        // Set Binding to value
        local_env
            .borrow_mut()
            .initialize_binding(param.name(), array);
    }

    // Adds an argument to the environment
    fn add_arguments_to_environment(
        &self,
        param: &FormalParameter,
        value: Value,
        local_env: &Environment,
    ) {
        // Create binding
        local_env
            .borrow_mut()
            .create_mutable_binding(param.name().to_owned(), false);

        // Set Binding to value
        local_env
            .borrow_mut()
            .initialize_binding(param.name(), value);
    }

    /// Returns true if the function object is callable.
    pub fn is_callable(&self) -> bool {
        self.callable
    }

    /// Returns true if the function object is constructable.
    pub fn is_constructable(&self) -> bool {
        self.constructable
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        write!(f, "[Not implemented]")?;
        write!(f, "}}")
    }
}

/// Arguments.
///
/// <https://tc39.es/ecma262/#sec-createunmappedargumentsobject>
pub fn create_unmapped_arguments_object(arguments_list: &[Value]) -> Value {
    let len = arguments_list.len();
    let mut obj = Object::default();
    obj.set_internal_slot("ParameterMap", Value::undefined());
    // Set length
    let mut length = Property::default();
    length = length.writable(true).value(Value::from(len));
    // Define length as a property
    obj.define_own_property("length".to_string(), length);
    let mut index: usize = 0;
    while index < len {
        let val = arguments_list.get(index).expect("Could not get argument");
        let mut prop = Property::default();
        prop = prop
            .value(val.clone())
            .enumerable(true)
            .writable(true)
            .configurable(true);

        obj.properties.insert(index.to_string(), prop);
        index += 1;
    }

    Value::from(obj)
}

/// Create new function `[[Construct]]`
///
// This gets called when a new Function() is created.
pub fn make_function(this: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    this.set_kind(ObjectKind::Function);
    Ok(this.clone())
}

pub fn create(global: &Value) -> Value {
    let prototype = Value::new_object(Some(global));

    make_constructor_fn("Function", 1, make_function, global, prototype, true)
}

/// Creates a new constructor function
///
/// This utility function handling linking the new Constructor to the prototype.
/// So far this is only used by internal functions
pub fn make_constructor_fn(
    name: &str,
    length: i32,
    body: NativeFunctionData,
    global: &Value,
    proto: Value,
    constructable: bool,
) -> Value {
    // Create the native function
    let mut constructor_fn = Function::builtin(Vec::new(), body);

    constructor_fn.constructable = constructable;

    // Get reference to Function.prototype
    let func_prototype = global.get_field("Function").get_field(PROTOTYPE);

    // Create the function object and point its instance prototype to Function.prototype
    let mut constructor_obj = Object::function();
    constructor_obj.set_func(constructor_fn);

    constructor_obj.set_internal_slot(INSTANCE_PROTOTYPE, func_prototype);
    let constructor_val = Value::from(constructor_obj);

    // Set proto.constructor -> constructor_obj
    proto.set_field("constructor", constructor_val.clone());
    constructor_val.set_field(PROTOTYPE, proto);

    let length = Property::new()
        .value(Value::from(length))
        .writable(false)
        .configurable(false)
        .enumerable(false);
    constructor_val.set_property_slice("length", length);

    let name = Property::new()
        .value(Value::from(name))
        .writable(false)
        .configurable(false)
        .enumerable(false);
    constructor_val.set_property_slice("name", name);

    constructor_val
}

/// Macro to create a new member function of a prototype.
///
/// If no length is provided, the length will be set to 0.
pub fn make_builtin_fn<N>(function: NativeFunctionData, name: N, parent: &Value, length: i32)
where
    N: Into<String>,
{
    let name_copy: String = name.into();
    let label = format!("{}{}", String::from("make_builtin_fn: "), &name_copy);
    let _timer = BoaProfiler::global().start_event(&label, "init");
    let func = Function::builtin(Vec::new(), function);

    let mut new_func = Object::function();
    new_func.set_func(func);

    let new_func_obj = Value::from(new_func);
    new_func_obj.set_field("length", length);

    parent.set_field(Value::from(name_copy), new_func_obj);
}

/// Initialise the `Function` object on the global object.
#[inline]
pub fn init(global: &Value) {
    let _timer = BoaProfiler::global().start_event("function", "init");
    global.set_field("Function", create(global));
}
