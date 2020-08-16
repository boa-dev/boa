use super::{Executable, Interpreter};
use crate::{builtins::Value, syntax::ast::node::If, Result};
use std::borrow::Borrow;

impl Executable for If {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        Ok(if self.cond().run(interpreter)?.borrow().to_boolean() {
            self.body().run(interpreter)?
        } else if let Some(ref else_e) = self.else_node() {
            else_e.run(interpreter)?
        } else {
            Value::undefined()
        })
    }
}
