use super::{Executable, Interpreter};
use crate::{
    builtins::value::{Type, Value},
    syntax::ast::node::{GetConstField, GetField},
    Result,
};

impl Executable for GetConstField {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        let mut obj = self.obj().run(interpreter)?;
        if obj.get_type() != Type::Object {
            obj = obj.to_object(interpreter)?;
        }

        Ok(obj.get_field(self.field()))
    }
}

impl Executable for GetField {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        let mut obj = self.obj().run(interpreter)?;
        if obj.get_type() != Type::Object {
            obj = obj.to_object(interpreter)?;
        }
        let field = self.field().run(interpreter)?;

        Ok(obj.get_field(field.to_property_key(interpreter)?))
    }
}
