use crate::{
    builtins::function::{ConstructorKind, Function},
    error::JsNativeError,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsValue,
};

pub(crate) mod field;
pub(crate) mod private;

pub(crate) use field::*;
pub(crate) use private::*;

/// `PushClassPrototype` implements the Opcode Operation for `Opcode::PushClassPrototype`
///
/// Operation:
///  - Get the prototype of a superclass and push it on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushClassPrototype;

impl Operation for PushClassPrototype {
    const NAME: &'static str = "PushClassPrototype";
    const INSTRUCTION: &'static str = "INST - PushClassPrototype";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let superclass = context.vm.pop();

        if let Some(superclass) = superclass.as_constructor() {
            let proto = superclass.get("prototype", context)?;
            if !proto.is_object() && !proto.is_null() {
                return Err(JsNativeError::typ()
                    .with_message("superclass prototype must be an object or null")
                    .into());
            }

            let class = context.vm.pop();
            {
                let class_object = class.as_object().expect("class must be object");
                class_object.set_prototype(Some(superclass.clone()));

                let mut class_object_mut = class_object.borrow_mut();
                let class_function = class_object_mut
                    .as_function_mut()
                    .expect("class must be function object");
                if let Function::Ordinary {
                    constructor_kind, ..
                } = class_function
                {
                    *constructor_kind = ConstructorKind::Derived;
                }
            }

            context.vm.push(class);
            context.vm.push(proto);
            Ok(ShouldExit::False)
        } else if superclass.is_null() {
            context.vm.push(JsValue::Null);
            Ok(ShouldExit::False)
        } else {
            Err(JsNativeError::typ()
                .with_message("superclass must be a constructor")
                .into())
        }
    }
}
