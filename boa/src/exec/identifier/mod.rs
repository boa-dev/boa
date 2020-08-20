use super::{Executable, Interpreter};
use crate::{builtins::value::Value, syntax::ast::node::identifier::Identifier, Result};

impl Executable for Identifier {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        interpreter
            .realm()
            .environment
            .get_binding_value(self.as_ref())
            .or_else(|e| Err(e.to_error(interpreter)))
    }
}
