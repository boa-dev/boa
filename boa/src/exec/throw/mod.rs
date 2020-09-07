use super::{Executable, Interpreter};
use crate::{syntax::ast::node::Throw, Result, Value};

impl Executable for Throw {
    #[inline]
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        Err(self.expr().run(interpreter)?)
    }
}
