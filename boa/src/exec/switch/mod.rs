use super::{Executable, Interpreter, InterpreterState};
use crate::{
    builtins::value::{ResultValue, Value},
    syntax::ast::node::Switch,
};

#[cfg(test)]
mod tests;

impl Executable for Switch {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let default = self.default();
        let val = self.val().run(interpreter)?;
        let mut result = Value::null();
        let mut matched = false;
        interpreter.set_current_state(InterpreterState::Executing);

        // If a case block does not end with a break statement then subsequent cases will be run without
        // checking their conditions until a break is encountered.
        let mut fall_through: bool = false;

        for case in self.cases().iter() {
            let cond = case.condition();
            let block = case.body();
            if fall_through || val.strict_equals(&cond.run(interpreter)?) {
                matched = true;
                block.run(interpreter)?;
                if interpreter.is_break() {
                    // Break statement encountered so therefore end switch statement.
                    break;
                } else {
                    fall_through = true;
                }
            }
        }
        if !matched {
            if let Some(default) = default {
                result = default.run(interpreter)?;
            }
        }
        Ok(result)
    }
}
