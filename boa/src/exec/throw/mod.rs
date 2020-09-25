use super::{Context, Executable};
use crate::{syntax::ast::node::Throw, Result, Value};

impl Executable for Throw {
    #[inline]
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        Err(self.expr().run(interpreter)?)
    }
}
