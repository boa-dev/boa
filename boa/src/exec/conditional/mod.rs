use super::{Executable, Interpreter};
use crate::{
    builtins::{ResultValue, Value},
    syntax::ast::node::If,
};
use std::borrow::Borrow;

impl Executable for If {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        Ok(if self.cond().run(interpreter)?.borrow().to_boolean() {
            self.body().run(interpreter)?
        } else if let Some(ref else_e) = self.else_node() {
            else_e.run(interpreter)?
        } else {
            Value::undefined()
        })
    }
}
