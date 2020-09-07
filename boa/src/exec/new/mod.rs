use super::{Executable, Interpreter};
use crate::{syntax::ast::node::New, BoaProfiler, Result, Value};

impl Executable for New {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("New", "exec");

        let func_object = self.expr().run(interpreter)?;
        let mut v_args = Vec::with_capacity(self.args().len());
        for arg in self.args() {
            v_args.push(arg.run(interpreter)?);
        }

        match func_object {
            Value::Object(ref object) => object.construct(&v_args, interpreter),
            _ => Ok(Value::undefined()),
        }
    }
}
