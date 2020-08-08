use super::{Executable, Interpreter};
use crate::{
    builtins::{object::PROTOTYPE, Value},
    syntax::ast::node::New,
    BoaProfiler, Result,
};

impl Executable for New {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("New", "exec");

        let func_object = self.expr().run(interpreter)?;
        let mut v_args = Vec::with_capacity(self.args().len());
        for arg in self.args() {
            v_args.push(arg.run(interpreter)?);
        }
        let this = Value::new_object(None);
        // Create a blank object, then set its __proto__ property to the [Constructor].prototype
        this.as_object_mut()
            .expect("this was not an object")
            .set_prototype(func_object.get_field(PROTOTYPE));

        match func_object {
            Value::Object(ref object) => object.construct(&this, &v_args, interpreter),
            _ => Ok(Value::undefined()),
        }
    }
}
