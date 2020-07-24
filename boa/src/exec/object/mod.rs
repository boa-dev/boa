//! Object execution.

use super::{Executable, Interpreter};
use crate::{
    builtins::value::{ResultValue, Value},
    syntax::ast::node::MethodDefinitionKind,
    syntax::ast::node::{Object, PropertyDefinition},
};

use std::borrow::Borrow;

impl Executable for Object {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let global_val = &interpreter
            .realm()
            .environment
            .get_global_object()
            .expect("Could not get the global object");
        let obj = Value::new_object(Some(global_val));

        // TODO: Implement the rest of the property types.
        for property in self.properties().iter() {
            match property {
                PropertyDefinition::Property(key, value) => {
                    obj.borrow().set_str_field(key, value.run(interpreter)?);
                }
                PropertyDefinition::MethodDefinition(kind, name, func) => {
                    if let MethodDefinitionKind::Ordinary = kind {
                        obj.borrow().set_str_field(name, func.run(interpreter)?);
                    } else {
                        // TODO: Implement other types of MethodDefinitionKinds.
                        unimplemented!("other types of property method definitions.");
                    }
                }
                i => unimplemented!("{:?} type of property", i),
            }
        }

        Ok(obj)
    }
}
