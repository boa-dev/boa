use gc::GcCell;
use js::object::ObjectData;
use js::value::{ResultValue, Value, ValueData};
use js::{function, json, object};
use std::cell::RefCell;
use syntax::ast::expr::Expr;

/// An execution engine
pub trait Executor {
    /// Make a new execution engine
    fn new() -> Self;
    /// Set a global variable called `name` with the value `val`
    fn set_global(&mut self, name: String, val: Value) -> Value;
    /// Resolve the global variable `name`
    fn get_global(&self, name: String) -> Value;
    /// Create a new scope and return it
    fn make_scope(&mut self) -> GcCell<RefCell<ObjectData>>;
    /// Destroy the current scope
    fn destroy_scope(&mut self) -> ();
    /// Run an expression
    fn run(&mut self, expr: &Expr) -> ResultValue;
}

/// A Javascript intepreter
pub struct Interpreter {
    /// An object representing the global object
    global: Value,
    /// The variable scopes
    scopes: Vec<GcCell<RefCell<ObjectData>>>,
}

impl Executor for Interpreter {
    fn new() -> Interpreter {
        let global = ValueData::new_obj(None);
        object::init(global);
        function::init(global);
        json::init(global);
        Interpreter {
            global: global,
            scopes: Vec::new(),
        }
    }
}
