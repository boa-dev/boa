use super::{Executable, Interpreter, InterpreterState};
use crate::{
    builtins::value::{ResultValue, Value},
    syntax::ast::node::Break,
};

impl Executable for Break {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        interpreter.set_current_state(InterpreterState::Break("".to_string()));
        Ok(Value::undefined())
    }
}
