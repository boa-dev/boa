use crate::environment::lexical_environment::Environment;
use crate::exec::Interpreter;
use crate::js::object::{Object, ObjectKind};
use crate::js::value::{Value, ValueData};
use crate::syntax::ast::expr::Expr;
/// Sets the functionKind
#[derive(Debug, Copy, Clone)]
pub enum FunctionKind {
    Normal,
    ClassConstructor,
    Generator,
    Async,
    AsyncGenerator,
    NonConstructor,
}
/// Sets the ConstructorKind
#[derive(Debug, Copy, Clone)]
pub enum ConstructorKind {
    Base,
    Derived,
}
/// Defines how this references are interpreted within the formal parameters and code body of the function.
#[derive(Debug, Copy, Clone)]
pub enum ThisMode {
    Lexical,
    Strict,
    Global,
}

/// Function Prototype
/// <https://tc39.es/ecma262/#sec-properties-of-the-function-prototype-object>
pub fn create_function_prototype() {
    let mut function_prototype: Object = Object::default();
    // Set Kind to function
    function_prototype.kind = ObjectKind::Function;
}

/// <https://tc39.es/ecma262/#sec-functionallocate>
pub fn function_allocate(proto: Value, mut kind: FunctionKind, ctx: &mut Interpreter) {
    let needs_construct: bool;

    match kind {
        FunctionKind::Normal => needs_construct = true,
        FunctionKind::NonConstructor => {
            needs_construct = false;
            kind = FunctionKind::Normal;
        }
        _ => needs_construct = false,
    }

    // Create new function object
    let mut func = Object::default();
    func.kind = ObjectKind::Function;
    // func.set_internal_method()
}

// <https://tc39.es/ecma262/#sec-ecmascript-function-objects-call-thisargument-argumentslist>
pub fn call(this: &Value, args: &[Value], callerContext: &mut Interpreter) {
    let calleeContext = Interpreter::new()
}
