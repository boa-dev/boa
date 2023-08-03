use crate::{
    js_string,
    object::PrivateElement,
    property::PropertyDescriptor,
    string::utf16,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `PushClassPrivateMethod` implements the Opcode Operation for `Opcode::PushClassPrivateMethod`
///
/// Operation:
///  - Push a private method to the class.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushClassPrivateMethod;

impl PushClassPrivateMethod {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let name = context.vm.frame().code_block.names[index].clone();
        let method = context.vm.pop();
        let method_object = method.as_callable().expect("method must be callable");

        let name_string = js_string!(utf16!("#"), &name);
        let desc = PropertyDescriptor::builder()
            .value(name_string)
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build();
        method_object
            .__define_own_property__(&utf16!("name").into(), desc, context)
            .expect("failed to set name property on private method");

        let class = context.vm.pop();
        let class_object = class.as_object().expect("class must be function object");

        class_object
            .borrow_mut()
            .as_function_mut()
            .expect("class must be function object")
            .push_private_method(
                class_object.private_name(name),
                PrivateElement::Method(method_object.clone()),
            );

        let mut method_object_mut = method_object.borrow_mut();
        let function = method_object_mut
            .as_function_mut()
            .expect("method must be function object");
        function.set_class_object(class_object.clone());
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushClassPrivateMethod {
    const NAME: &'static str = "PushClassPrivateMethod";
    const INSTRUCTION: &'static str = "INST - PushClassPrivateMethod";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn half_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn wide_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}

/// `PushClassPrivateGetter` implements the Opcode Operation for `Opcode::PushClassPrivateGetter`
///
/// Operation:
///  - Push a private getter to the class.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushClassPrivateGetter;

impl PushClassPrivateGetter {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let name = context.vm.frame().code_block.names[index].clone();
        let getter = context.vm.pop();
        let getter_object = getter.as_callable().expect("getter must be callable");
        let class = context.vm.pop();
        let class_object = class.as_object().expect("class must be function object");

        class_object
            .borrow_mut()
            .as_function_mut()
            .expect("class must be function object")
            .push_private_method(
                class_object.private_name(name),
                PrivateElement::Accessor {
                    getter: Some(getter_object.clone()),
                    setter: None,
                },
            );
        let mut getter_object_mut = getter_object.borrow_mut();
        let function = getter_object_mut
            .as_function_mut()
            .expect("getter must be function object");
        function.set_class_object(class_object.clone());
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushClassPrivateGetter {
    const NAME: &'static str = "PushClassPrivateGetter";
    const INSTRUCTION: &'static str = "INST - PushClassPrivateGetter";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn half_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn wide_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}

/// `PushClassPrivateSetter` implements the Opcode Operation for `Opcode::PushClassPrivateSetter`
///
/// Operation:
///  - Push a private setter to the class.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushClassPrivateSetter;

impl PushClassPrivateSetter {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let name = context.vm.frame().code_block.names[index].clone();
        let setter = context.vm.pop();
        let setter_object = setter.as_callable().expect("getter must be callable");
        let class = context.vm.pop();
        let class_object = class.as_object().expect("class must be function object");

        class_object
            .borrow_mut()
            .as_function_mut()
            .expect("class must be function object")
            .push_private_method(
                class_object.private_name(name),
                PrivateElement::Accessor {
                    getter: None,
                    setter: Some(setter_object.clone()),
                },
            );
        let mut setter_object_mut = setter_object.borrow_mut();
        let function = setter_object_mut
            .as_function_mut()
            .expect("setter must be function object");
        function.set_class_object(class_object.clone());
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushClassPrivateSetter {
    const NAME: &'static str = "PushClassPrivateSetter";
    const INSTRUCTION: &'static str = "INST - PushClassPrivateSetter";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn half_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn wide_execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}
