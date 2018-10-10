use js::object::{INSTANCE_PROTOTYPE, PROTOTYPE, ObjectData};

/// An execution engine
pub trait Executor {
    /// Make a new execution engine
    fn new() -> Self;
    /// Set a global variable called `name` with the value `val`
    // fn set_global(&mut self, name: String, val: Value) -> Value;
    /// Resolve the global variable `name`
    // fn get_global(&self, name: String) -> Value;
    /// Create a new scope and return it
    // fn make_scope(&mut self) -> Gc<RefCell<ObjectData>>;
    /// Destroy the current scope
    fn destroy_scope(&mut self) -> ();
    /// Run an expression
    // fn run(&mut self, expr: &Expr) -> ResultValue;
}
