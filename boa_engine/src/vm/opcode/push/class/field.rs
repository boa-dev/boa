use crate::{
    object::JsFunction,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `PushClassField` implements the Opcode Operation for `Opcode::PushClassField`
///
/// Operation:
///  - Push a field to a class.
#[derive(Debug, Clone, Copy)]
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

/// `PushClassFieldPrivate` implements the Opcode Operation for `Opcode::PushClassFieldPrivate`
///
/// Operation:
///  - Push a private field to the class.
#[derive(Debug, Clone, Copy)]
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
                name.sym(),
                JsFunction::from_object_unchecked(field_function_object.clone()),
            );
        Ok(ShouldExit::False)
    }
}
