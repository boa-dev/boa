//! Expression execution.

use super::{Executable, Interpreter};
use crate::{
    builtins::{
        object::{INSTANCE_PROTOTYPE, PROTOTYPE},
        value::{ResultValue, Value, ValueData},
    },
    syntax::ast::node::{Call, New, Node},
    BoaProfiler,
};

impl Executable for Call {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let _timer = BoaProfiler::global().start_event("Call", "exec");
        let (mut this, func) = match self.expr() {
            Node::GetConstField(ref obj, ref field) => {
                let mut obj = obj.run(interpreter)?;
                if obj.get_type() != "object" || obj.get_type() != "symbol" {
                    obj = interpreter
                        .to_object(&obj)
                        .expect("failed to convert to object");
                }
                (obj.clone(), obj.get_field(field))
            }
            Node::GetField(ref obj, ref field) => {
                let obj = obj.run(interpreter)?;
                let field = field.run(interpreter)?;
                (obj.clone(), obj.get_field(field))
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
        let fnct_result = interpreter.call(&func, &mut this, &v_args);

        // unset the early return flag
        interpreter.is_return = false;

        fnct_result
    }
}

impl Executable for New {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        // let (callee, args) = match call.as_ref() {
        //     Node::Call(callee, args) => (callee, args),
        //     _ => unreachable!("Node::New(ref call): 'call' must only be Node::Call type."),
        // };

        let func_object = self.expr().run(interpreter)?;
        let mut v_args = Vec::with_capacity(self.args().len());
        for arg in self.args() {
            v_args.push(arg.run(interpreter)?);
        }
        let mut this = Value::new_object(None);
        // Create a blank object, then set its __proto__ property to the [Constructor].prototype
        this.set_internal_slot(INSTANCE_PROTOTYPE, func_object.get_field(PROTOTYPE));

        match func_object.data() {
            ValueData::Object(ref o) => o.clone().borrow_mut().func.as_ref().unwrap().construct(
                &mut func_object.clone(),
                &v_args,
                interpreter,
                &mut this,
            ),
            _ => Ok(Value::undefined()),
        }
    }
}
