use super::{Executable, Interpreter};
use crate::{
    builtins::value::{ResultValue, Value, ValueData},
    syntax::ast::node::identifier::Identifier,
};

impl Executable for Identifier {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let reference = resolve_binding(interpreter, self.as_ref());
        match reference.data() {
            ValueData::Undefined => Err(interpreter
                .throw_reference_error(self.as_ref())
                .expect_err("throw_reference_error() must return an error")),
            _ => Ok(reference),
        }
    }
}

pub(crate) fn resolve_binding(interpreter: &mut Interpreter, name: &str) -> Value {
    interpreter.realm().environment.get_binding_value(name)
}
