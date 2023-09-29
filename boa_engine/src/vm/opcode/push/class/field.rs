use crate::{
    object::JsFunction,
    vm::{opcode::Operation, CompletionType},
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

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
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
        field_function.set_class_object(class_object.clone());
        class_object
            .borrow_mut()
            .as_function_mut()
            .expect("class must be function object")
            .push_field(
                field_name_key,
                JsFunction::from_object_unchecked(field_function_object.clone()),
            );
        Ok(CompletionType::Normal)
    }
}

/// `PushClassFieldPrivate` implements the Opcode Operation for `Opcode::PushClassFieldPrivate`
///
/// Operation:
///  - Push a private field to the class.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushClassFieldPrivate;

impl PushClassFieldPrivate {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let name = context.vm.frame().code_block.names[index].clone();
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
        field_function.set_class_object(class_object.clone());

        class_object
            .borrow_mut()
            .as_function_mut()
            .expect("class must be function object")
            .push_field_private(
                class_object.private_name(name),
                JsFunction::from_object_unchecked(field_function_object.clone()),
            );
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushClassFieldPrivate {
    const NAME: &'static str = "PushClassFieldPrivate";
    const INSTRUCTION: &'static str = "INST - PushClassFieldPrivate";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn u16_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn u32_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}
