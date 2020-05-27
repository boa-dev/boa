use super::{Executable, Interpreter};
use crate::{
    builtins::{
        object::{INSTANCE_PROTOTYPE, PROTOTYPE},
        value::{ResultValue, Value, ValueData},
    },
    syntax::ast::node::{Call, New, Node},
};

impl Executable for GetConstField {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let mut obj = obj.run(interpreter)?;
        if obj.get_type() != "object" || obj.get_type() != "symbol" {
            obj = interpreter
                .to_object(&obj)
                .expect("failed to convert to object");
        }
        (obj.clone(), obj.get_field(field))
    }
}



