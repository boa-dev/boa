use crate::{
    builtins::function::{ConstructorKind, Function},
    object::{JsFunction, PrivateElement},
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsValue,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PushClassField;

impl Operation for PushClassField {
    const NAME: &'static str = "PushClassField";
    const INSTRUCTION: &'static str = "INST - PushClassField";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let field_function_value = context.vm.pop();
        let field_name_value = context.vm.pop();
        let class_value = context.vm.pop();

        let field_name_key = field_name_value.to_property_key(context)?;
        let field_function_object = field_function_value
            .as_object()
            .expect("field value must be function object");
        let mut field_function_object_borrow = field_function_object.borrow_mut();
        let field_function = field_function_object_borrow
            .as_function_mut()
            .expect("field value must be function object");
        let class_object = class_value
            .as_object()
            .expect("class must be function object");
        field_function.set_home_object(class_object.clone());
        class_object
            .borrow_mut()
            .as_function_mut()
            .expect("class must be function object")
            .push_field(
                field_name_key,
                JsFunction::from_object_unchecked(field_function_object.clone()),
            );
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PushClassFieldPrivate;

impl Operation for PushClassFieldPrivate {
    const NAME: &'static str = "PushClassFieldPrivate";
    const INSTRUCTION: &'static str = "INST - PushClassFieldPrivate";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code.names[index as usize];
        let field_function_value = context.vm.pop();
        let class_value = context.vm.pop();

        let field_function_object = field_function_value
            .as_object()
            .expect("field value must be function object");
        let mut field_function_object_borrow = field_function_object.borrow_mut();
        let field_function = field_function_object_borrow
            .as_function_mut()
            .expect("field value must be function object");
        let class_object = class_value
            .as_object()
            .expect("class must be function object");
        field_function.set_home_object(class_object.clone());
        class_object
            .borrow_mut()
            .as_function_mut()
            .expect("class must be function object")
            .push_field_private(
                name,
                JsFunction::from_object_unchecked(field_function_object.clone()),
            );
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PushClassPrivateMethod;

impl Operation for PushClassPrivateMethod {
    const NAME: &'static str = "PushClassPrivateMethod";
    const INSTRUCTION: &'static str = "INST - PushClassPrivateMethod";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code.names[index as usize];
        let method = context.vm.pop();
        let method_object = method.as_callable().expect("method must be callable");
        let class = context.vm.pop();
        class
            .as_object()
            .expect("class must be function object")
            .borrow_mut()
            .as_function_mut()
            .expect("class must be function object")
            .push_private_method(name, PrivateElement::Method(method_object.clone()));
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PushClassPrivateGetter;

impl Operation for PushClassPrivateGetter {
    const NAME: &'static str = "PushClassPrivateGetter";
    const INSTRUCTION: &'static str = "INST - PushClassPrivateGetter";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code.names[index as usize];
        let getter = context.vm.pop();
        let getter_object = getter.as_callable().expect("getter must be callable");
        let class = context.vm.pop();
        class
            .as_object()
            .expect("class must be function object")
            .borrow_mut()
            .as_function_mut()
            .expect("class must be function object")
            .push_private_method(
                name,
                PrivateElement::Accessor {
                    getter: Some(getter_object.clone()),
                    setter: None,
                },
            );
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PushClassPrivateSetter;

impl Operation for PushClassPrivateSetter {
    const NAME: &'static str = "PushClassPrivateSetter";
    const INSTRUCTION: &'static str = "INST - PushClassPrivateSetter";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code.names[index as usize];
        let setter = context.vm.pop();
        let setter_object = setter.as_callable().expect("getter must be callable");
        let class = context.vm.pop();
        class
            .as_object()
            .expect("class must be function object")
            .borrow_mut()
            .as_function_mut()
            .expect("class must be function object")
            .push_private_method(
                name,
                PrivateElement::Accessor {
                    getter: None,
                    setter: Some(setter_object.clone()),
                },
            );
        Ok(ShouldExit::False)
    }
}
