use super::{Executable, Interpreter};
use crate::{
    builtins::value::{ResultValue, Value},
    syntax::ast::node::Switch,
};

impl Executable for Switch {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let default = self.default();
        let val = self.val().run(interpreter)?;
        let mut result = Value::null();
        let mut matched = false;
        for case in self.cases().iter() {
            let cond = case.condition();
            let block = case.body();
            if val.strict_equals(&cond.run(interpreter)?) {
                matched = true;
                block.run(interpreter)?;
            }

            // TODO: break out of switch on a break statement.
        }
        if !matched {
            if let Some(default) = default {
                result = default.run(interpreter)?;
            }
        }
        Ok(result)
    }
}
