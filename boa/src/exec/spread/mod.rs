use super::{Executable, Interpreter};
use crate::{builtins::value::ResultValue, syntax::ast::node::Spread};

impl Executable for Spread {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        // TODO: for now we can do nothing but return the value as-is
        self.val().run(interpreter)
    }
}
