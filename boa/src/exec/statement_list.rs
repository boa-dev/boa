//! Statement list execution.

use super::{Executable, Interpreter};
use crate::{
    builtins::value::{ResultValue, Value},
    syntax::ast::node::StatementList,
};

impl Executable for StatementList {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let mut obj = Value::null();
        for (i, item) in self.statements().iter().enumerate() {
            let val = item.run(interpreter)?;
            // early return
            if interpreter.is_return {
                obj = val;
                break;
            }
            if i + 1 == self.statements().len() {
                obj = val;
            }
        }

        Ok(obj)
    }
}
