use super::{Executable, Interpreter};
use crate::{
    builtins::{ResultValue, Value},
    syntax::ast::node::If,
};
use std::borrow::Borrow;

impl Executable for If {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        Ok(if self.cond().run(interpreter)?.borrow().is_true() {
            self.body().run(interpreter)?
        } else {
            match self.else_node() {
                Some(ref else_e) => else_e.run(interpreter)?,
                None => Value::undefined(),
            }
        })
    }
}
