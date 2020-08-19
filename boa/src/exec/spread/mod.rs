use super::{Executable, Interpreter};
use crate::{builtins::value::Value, syntax::ast::node::Spread, Result};

impl Executable for Spread {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        // TODO: for now we can do nothing but return the value as-is
        self.val().run(interpreter)
    }
}
