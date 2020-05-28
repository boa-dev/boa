use super::{Executable, Interpreter};
use crate::{
    builtins::value::ResultValue,
    syntax::ast::node::GetConstField,
};

impl Executable for GetConstField {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let mut obj = self.obj().run(interpreter)?;
        if obj.get_type() != "object" || obj.get_type() != "symbol" {
            obj = interpreter
                .to_object(&obj)
                .expect("failed to convert to object");
        }


        Ok(obj.get_field(self.field()))
    }
}



