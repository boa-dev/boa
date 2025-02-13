use crate::{
    builtins::function::OrdinaryFunction,
    js_str, js_string,
    object::{internal_methods::InternalMethodContext, PrivateElement},
    property::PropertyDescriptor,
    vm::{opcode::Operation, CompletionType, Registers},
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
    fn operation(
        object: u32,
        prototype: u32,
        value: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let object = registers.get(object);
        let prototype = registers.get(prototype);
        let value = registers.get(value);
        let name = context.vm.frame().code_block().constant_string(index);

        let value = value.as_callable().expect("method must be callable");
        let prototype = prototype
            .as_object()
            .expect("class_prototype must be function object");
        let object = object.as_object().expect("class must be function object");

        let name_string = js_string!(js_str!("#"), &name);
        let desc = PropertyDescriptor::builder()
            .value(name_string)
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build();
        value
            .__define_own_property__(
                &js_string!("name").into(),
                desc,
                &mut InternalMethodContext::new(context),
            )
            .expect("failed to set name property on private method");
        value
            .downcast_mut::<OrdinaryFunction>()
            .expect("method must be function object")
            .set_home_object(prototype.clone());

        object
            .downcast_mut::<OrdinaryFunction>()
            .expect("class must be function object")
            .push_private_method(
                object.private_name(name),
                PrivateElement::Method(value.clone()),
            );

        Ok(CompletionType::Normal)
    }
}

impl Operation for PushClassPrivateMethod {
    const NAME: &'static str = "PushClassPrivateMethod";
    const INSTRUCTION: &'static str = "INST - PushClassPrivateMethod";
    const COST: u8 = 6;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u8>().into();
        let prototype = context.vm.read::<u8>().into();
        let value = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(object, prototype, value, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u16>().into();
        let prototype = context.vm.read::<u16>().into();
        let value = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(object, prototype, value, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u32>();
        let prototype = context.vm.read::<u32>();
        let value = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(object, prototype, value, index, registers, context)
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
    fn operation(
        object: u32,
        value: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let object = registers.get(object);
        let value = registers.get(value);
        let name = context.vm.frame().code_block().constant_string(index);

        let value = value.as_callable().expect("getter must be callable");
        let object = object.as_object().expect("class must be function object");

        object
            .downcast_mut::<OrdinaryFunction>()
            .expect("class must be function object")
            .push_private_method(
                object.private_name(name),
                PrivateElement::Accessor {
                    getter: Some(value.clone()),
                    setter: None,
                },
            );

        Ok(CompletionType::Normal)
    }
}

impl Operation for PushClassPrivateGetter {
    const NAME: &'static str = "PushClassPrivateGetter";
    const INSTRUCTION: &'static str = "INST - PushClassPrivateGetter";
    const COST: u8 = 6;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u8>().into();
        let value = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(object, value, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u16>().into();
        let value = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(object, value, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u32>();
        let value = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(object, value, index, registers, context)
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
    fn operation(
        object: u32,
        value: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let object = registers.get(object);
        let value = registers.get(value);
        let name = context.vm.frame().code_block().constant_string(index);

        let value = value.as_callable().expect("getter must be callable");
        let object = object.as_object().expect("class must be function object");

        object
            .downcast_mut::<OrdinaryFunction>()
            .expect("class must be function object")
            .push_private_method(
                object.private_name(name),
                PrivateElement::Accessor {
                    getter: None,
                    setter: Some(value.clone()),
                },
            );

        Ok(CompletionType::Normal)
    }
}

impl Operation for PushClassPrivateSetter {
    const NAME: &'static str = "PushClassPrivateSetter";
    const INSTRUCTION: &'static str = "INST - PushClassPrivateSetter";
    const COST: u8 = 6;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u8>().into();
        let value = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(object, value, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u16>().into();
        let value = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(object, value, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u32>();
        let value = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(object, value, index, registers, context)
    }
}
