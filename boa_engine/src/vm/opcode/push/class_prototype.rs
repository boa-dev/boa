use crate::{
    vm::{ShouldExit, opcode::Operation},
    builtins::function::{Function, ConstructorKind},
    Context, JsValue, JsResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PushClassPrototype;

impl Operation for PushClassPrototype {
    const NAME: &'static str = "PushClassPrototype";
    const INSTRUCTION: &'static str = "INST - PushClassPrototype";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let superclass = context.vm.pop();

        if let Some(superclass) = superclass.as_constructor() {
            let proto = superclass.get("prototype", context)?;
            if !proto.is_object() && !proto.is_null() {
                return context.throw_type_error("superclass prototype must be an object or null");
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
            return context.throw_type_error("superclass must be a constructor");
        }
    }
}