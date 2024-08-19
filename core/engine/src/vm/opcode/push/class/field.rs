use crate::{
    builtins::function::OrdinaryFunction,
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
    const COST: u8 = 6;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let is_annonymus_function = context.vm.read::<u8>() != 0;
        let field_function_value = context.vm.pop();
        let field_name_value = context.vm.pop();
        let class_value = context.vm.pop();

        let field_name_key = field_name_value.to_property_key(context)?;
        let field_function_object = field_function_value
            .as_object()
            .expect("field value must be function object");
        let class_object = class_value
            .as_object()
            .expect("class must be function object");

        field_function_object
            .downcast_mut::<OrdinaryFunction>()
            .expect("field value must be function object")
            .set_home_object(class_object.clone());

        class_object
            .downcast_mut::<OrdinaryFunction>()
            .expect("class must be function object")
            .push_field(
                field_name_key.clone(),
                JsFunction::from_object_unchecked(field_function_object.clone()),
                if is_annonymus_function {
                    Some(field_name_key)
                } else {
                    None
                },
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
    fn operation(context: &mut Context, index: usize) -> JsResult<CompletionType> {
        let name = context.vm.frame().code_block().constant_string(index);
        let field_function_value = context.vm.pop();
        let class_value = context.vm.pop();

        let field_function_object = field_function_value
            .as_object()
            .expect("field value must be function object");
        let class_object = class_value
            .as_object()
            .expect("class must be function object");

        field_function_object
            .downcast_mut::<OrdinaryFunction>()
            .expect("field value must be function object")
            .set_home_object(class_object.clone());

        class_object
            .downcast_mut::<OrdinaryFunction>()
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
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}
