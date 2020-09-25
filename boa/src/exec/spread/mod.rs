use super::{Context, Executable};
use crate::{syntax::ast::node::Spread, Result, Value};

impl Executable for Spread {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        // TODO: for now we can do nothing but return the value as-is
        self.val().run(interpreter)
    }
}
