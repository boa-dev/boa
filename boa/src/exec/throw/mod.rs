use super::{Executable, Interpreter};
use crate::{builtins::value::Value, syntax::ast::node::Throw, Result};

impl Executable for Throw {
    #[inline]
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        Err(self.expr().run(interpreter)?)
    }
}
