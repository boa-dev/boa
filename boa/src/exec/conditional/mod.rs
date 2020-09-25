use super::{Context, Executable};
use crate::{
    syntax::ast::node::{ConditionalOp, If},
    Result, Value,
};

impl Executable for If {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        Ok(if self.cond().run(interpreter)?.to_boolean() {
            self.body().run(interpreter)?
        } else if let Some(ref else_e) = self.else_node() {
            else_e.run(interpreter)?
        } else {
            Value::undefined()
        })
    }
}

impl Executable for ConditionalOp {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        Ok(if self.cond().run(interpreter)?.to_boolean() {
            self.if_true().run(interpreter)?
        } else {
            self.if_false().run(interpreter)?
        })
    }
}
