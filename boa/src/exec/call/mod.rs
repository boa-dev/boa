use super::{Executable, Interpreter, InterpreterState};
use crate::{
    builtins::value::{Type, Value},
    syntax::ast::node::{Call, Node},
    BoaProfiler, Result,
};

impl Executable for Call {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("Call", "exec");
        let (this, func) = match self.expr() {
            Node::GetConstField(ref get_const_field) => {
                let mut obj = get_const_field.obj().run(interpreter)?;
                if obj.get_type() != Type::Object {
                    obj = obj.to_object(interpreter)?;
                }
                (obj.clone(), obj.get_field(get_const_field.field()))
            }
            Node::GetField(ref get_field) => {
                let obj = get_field.obj().run(interpreter)?;
                let field = get_field.field().run(interpreter)?;
                (
                    obj.clone(),
                    obj.get_field(field.to_property_key(interpreter)?),
                )
            }
            _ => (
                interpreter.realm().global_obj.clone(),
                self.expr().run(interpreter)?,
            ), // 'this' binding should come from the function's self-contained environment
        };
        let mut v_args = Vec::with_capacity(self.args().len());
        for arg in self.args() {
            if let Node::Spread(ref x) = arg {
                let val = x.run(interpreter)?;
                let mut vals = interpreter.extract_array_properties(&val).unwrap();
                v_args.append(&mut vals);
                break; // after spread we don't accept any new arguments
            }
            v_args.push(arg.run(interpreter)?);
        }

        // execute the function call itself
        let fnct_result = interpreter.call(&func, &this, &v_args);

        // unset the early return flag
        interpreter.set_current_state(InterpreterState::Executing);

        fnct_result
    }
}
