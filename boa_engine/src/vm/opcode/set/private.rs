use crate::{
    js_string,
    object::PrivateElement,
    property::PropertyDescriptor,
    string::utf16,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `SetPrivateField` implements the Opcode Operation for `Opcode::SetPrivateField`
///
/// Operation:
///  - Assign the value of a private property of an object by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrivateField;

impl SetPrivateField {
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let name = context.vm.frame().code_block.names[index].clone();
        let value = context.vm.pop();
        let object = context.vm.pop();
        let base_obj = object.to_object(context)?;

        let name = context
            .vm
            .environments
            .resolve_private_identifier(name)
            .expect("private name must be in environment");

        base_obj.private_set(&name, value.clone(), context)?;
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for SetPrivateField {
    const NAME: &'static str = "SetPrivateField";
    const INSTRUCTION: &'static str = "INST - SetPrivateField";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}

/// `DefinePrivateField` implements the Opcode Operation for `Opcode::DefinePrivateField`
///
/// Operation:
///  - Set a private property of a class constructor by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefinePrivateField;

impl DefinePrivateField {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let name = context.vm.frame().code_block.names[index].clone();
        let value = context.vm.pop();
        let object = context.vm.pop();
        let object = object
            .as_object()
            .expect("class prototype must be an object");

        object
            .borrow_mut()
            .append_private_element(object.private_name(name), PrivateElement::Field(value));

        Ok(CompletionType::Normal)
    }
}

impl Operation for DefinePrivateField {
    const NAME: &'static str = "DefinePrivateField";
    const INSTRUCTION: &'static str = "INST - DefinePrivateField";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}

/// `SetPrivateMethod` implements the Opcode Operation for `Opcode::SetPrivateMethod`
///
/// Operation:
///  - Set a private method of a class constructor by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrivateMethod;

impl SetPrivateMethod {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let name = context.vm.frame().code_block.names[index].clone();
        let value = context.vm.pop();
        let value = value.as_callable().expect("method must be callable");

        let name_string = js_string!(utf16!("#"), &name);
        let desc = PropertyDescriptor::builder()
            .value(name_string)
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build();
        value
            .__define_own_property__(&js_string!("name").into(), desc, context)
            .expect("failed to set name property on private method");

        let object = context.vm.pop();
        let object = object
            .as_object()
            .expect("class prototype must be an object");

        object.borrow_mut().append_private_element(
            object.private_name(name),
            PrivateElement::Method(value.clone()),
        );

        Ok(CompletionType::Normal)
    }
}

impl Operation for SetPrivateMethod {
    const NAME: &'static str = "SetPrivateMethod";
    const INSTRUCTION: &'static str = "INST - SetPrivateMethod";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}

/// `SetPrivateSetter` implements the Opcode Operation for `Opcode::SetPrivateSetter`
///
/// Operation:
///  - Set a private setter property of a class constructor by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrivateSetter;

impl SetPrivateSetter {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let name = context.vm.frame().code_block.names[index].clone();
        let value = context.vm.pop();
        let value = value.as_callable().expect("setter must be callable");
        let object = context.vm.pop();
        let object = object
            .as_object()
            .expect("class prototype must be an object");

        object.borrow_mut().append_private_element(
            object.private_name(name),
            PrivateElement::Accessor {
                getter: None,
                setter: Some(value.clone()),
            },
        );

        Ok(CompletionType::Normal)
    }
}

impl Operation for SetPrivateSetter {
    const NAME: &'static str = "SetPrivateSetter";
    const INSTRUCTION: &'static str = "INST - SetPrivateSetter";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}

/// `SetPrivateGetter` implements the Opcode Operation for `Opcode::SetPrivateGetter`
///
/// Operation:
///  - Set a private getter property of a class constructor by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrivateGetter;

impl SetPrivateGetter {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let name = context.vm.frame().code_block.names[index].clone();
        let value = context.vm.pop();
        let value = value.as_callable().expect("getter must be callable");
        let object = context.vm.pop();
        let object = object
            .as_object()
            .expect("class prototype must be an object");

        object.borrow_mut().append_private_element(
            object.private_name(name),
            PrivateElement::Accessor {
                getter: Some(value.clone()),
                setter: None,
            },
        );

        Ok(CompletionType::Normal)
    }
}

impl Operation for SetPrivateGetter {
    const NAME: &'static str = "SetPrivateGetter";
    const INSTRUCTION: &'static str = "INST - SetPrivateGetter";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}
