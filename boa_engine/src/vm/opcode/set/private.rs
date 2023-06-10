use crate::{
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

impl Operation for SetPrivateField {
    const NAME: &'static str = "SetPrivateField";
    const INSTRUCTION: &'static str = "INST - SetPrivateField";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code_block.names[index as usize].clone();
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

/// `DefinePrivateField` implements the Opcode Operation for `Opcode::DefinePrivateField`
///
/// Operation:
///  - Set a private property of a class constructor by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefinePrivateField;

impl Operation for DefinePrivateField {
    const NAME: &'static str = "DefinePrivateField";
    const INSTRUCTION: &'static str = "INST - DefinePrivateField";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code_block.names[index as usize].clone();
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

/// `SetPrivateMethod` implements the Opcode Operation for `Opcode::SetPrivateMethod`
///
/// Operation:
///  - Set a private method of a class constructor by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrivateMethod;

impl Operation for SetPrivateMethod {
    const NAME: &'static str = "SetPrivateMethod";
    const INSTRUCTION: &'static str = "INST - SetPrivateMethod";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code_block.names[index as usize].clone();
        let value = context.vm.pop();
        let value = value.as_callable().expect("method must be callable");

        let name_string = format!("#{}", name.to_std_string_escaped());
        let desc = PropertyDescriptor::builder()
            .value(name_string)
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build();
        value
            .__define_own_property__(&utf16!("name").into(), desc, context)
            .expect("failed to set name property on private method");

        let object = context.vm.pop();
        let object = object
            .as_object()
            .expect("class prototype must be an object");

        object.borrow_mut().append_private_element(
            object.private_name(name),
            PrivateElement::Method(value.clone()),
        );
        let mut value_mut = value.borrow_mut();
        let function = value_mut
            .as_function_mut()
            .expect("method must be a function");
        function.set_class_object(object.clone());

        Ok(CompletionType::Normal)
    }
}

/// `SetPrivateSetter` implements the Opcode Operation for `Opcode::SetPrivateSetter`
///
/// Operation:
///  - Set a private setter property of a class constructor by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrivateSetter;

impl Operation for SetPrivateSetter {
    const NAME: &'static str = "SetPrivateSetter";
    const INSTRUCTION: &'static str = "INST - SetPrivateSetter";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code_block.names[index as usize].clone();
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
        let mut value_mut = value.borrow_mut();
        let function = value_mut
            .as_function_mut()
            .expect("method must be a function");
        function.set_class_object(object.clone());

        Ok(CompletionType::Normal)
    }
}

/// `SetPrivateGetter` implements the Opcode Operation for `Opcode::SetPrivateGetter`
///
/// Operation:
///  - Set a private getter property of a class constructor by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrivateGetter;

impl Operation for SetPrivateGetter {
    const NAME: &'static str = "SetPrivateGetter";
    const INSTRUCTION: &'static str = "INST - SetPrivateGetter";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code_block.names[index as usize].clone();
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
        let mut value_mut = value.borrow_mut();
        let function = value_mut
            .as_function_mut()
            .expect("method must be a function");
        function.set_class_object(object.clone());

        Ok(CompletionType::Normal)
    }
}
