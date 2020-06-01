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
        for tup in self.cases().iter() {
            let cond = &tup.0;
            let block = &tup.1;
            if val.strict_equals(&cond.run(interpreter)?) {
                matched = true;
                let last_expr = block.last().expect("Block has no expressions");
                for expr in block.iter() {
                    let e_result = expr.run(interpreter)?;
                    if expr == last_expr {
                        result = e_result;
                    }
                }
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
