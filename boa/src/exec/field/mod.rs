use super::{Executable, Interpreter};
use crate::{
    builtins::value::{ResultValue, Type},
    syntax::ast::node::{GetConstField, GetField},
};

impl Executable for GetConstField {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let mut obj = self.obj().run(interpreter)?;
        if obj.get_type() != Type::Object || obj.get_type() != Type::Symbol {
            obj = interpreter
                .to_object(&obj)
                .expect("failed to convert to object");
        }

        Ok(obj.get_field(self.field()))
    }
}

impl Executable for GetField {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let obj = self.obj().run(interpreter)?;
        let field = self.field().run(interpreter)?;

        Ok(obj.get_field(field.to_string()))
    }
}
