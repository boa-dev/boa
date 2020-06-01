use super::{Executable, Interpreter};
use crate::{builtins::value::ResultValue, syntax::ast::node::Throw};

impl Executable for Throw {
    #[inline]
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        Err(self.expr().run(interpreter)?)
    }
}
