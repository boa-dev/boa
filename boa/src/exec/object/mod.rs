//! Object execution.

use super::{Executable, Interpreter};
use crate::{
    builtins::V
    builtins::value::ResultValue,
    environment::lexical_environment::{new_declarative_environment, VariableScope},
    syntax::ast::node::{Object, PropertyDefinition},
};

impl Executable for Object {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let global_val = &interpreter
            .realm()
            .environment
            .get_global_object()
            .expect("Could not get the global object");
        let obj = Value::new_object(Some(global_val));

        // TODO: Implement the rest of the property types.
        for property in properties.iter() {
            match property {
                PropertyDefinition::Property(key, value) => {
                    obj.borrow()
                        .set_field(&key.clone(), value.run(interpreter)?);
                }
                PropertyDefinition::MethodDefinition(kind, name, func) => {
                    if let MethodDefinitionKind::Ordinary = kind {
                        obj.borrow()
                            .set_field(&name.clone(), func.run(interpreter)?);
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