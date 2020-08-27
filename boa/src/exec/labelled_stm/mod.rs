use crate::{builtins::Value, syntax::ast::node::Label, Executable, Interpreter, Result};

impl Executable for Label {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        self.stmt.run(interpreter)
    }
}
