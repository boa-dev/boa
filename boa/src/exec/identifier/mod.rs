use super::{Executable, Interpreter};
use crate::{builtins::value::ResultValue, syntax::ast::node::identifier::Identifier};

impl Executable for Identifier {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let reference = interpreter
            .realm()
            .environment
            .get_binding_value(self.as_ref());
        match reference {
            Some(value) => Ok(value),
            None => Err(interpreter
                .throw_reference_error(self.as_ref())
                .expect_err("throw_reference_error() must return an error")),
        }
    }
}
