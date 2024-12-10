use crate::{
    builtins::function::OrdinaryFunction,
    object::JsFunction,
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

/// `PushClassField` implements the Opcode Operation for `Opcode::PushClassField`
///
/// Operation:
///  - Push a field to a class.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushClassField;

impl PushClassField {
    fn operation(
        class: u32,
        name: u32,
        function: u32,
        is_anonyms_function: bool,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let class = registers.get(class);
        let name = registers.get(name);
        let function = registers.get(function);

        let name = name.to_property_key(context)?;
        let function = function
            .as_object()
            .expect("field value must be function object");
        let class = class.as_object().expect("class must be function object");

        function
            .downcast_mut::<OrdinaryFunction>()
            .expect("field value must be function object")
            .set_home_object(class.clone());

        class
            .downcast_mut::<OrdinaryFunction>()
            .expect("class must be function object")
            .push_field(
                name.clone(),
                JsFunction::from_object_unchecked(function.clone()),
                if is_anonyms_function {
                    Some(name)
                } else {
                    None
                },
            );
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushClassField {
    const NAME: &'static str = "PushClassField";
    const INSTRUCTION: &'static str = "INST - PushClassField";
    const COST: u8 = 6;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let class = context.vm.read::<u8>().into();
        let name = context.vm.read::<u8>().into();
        let function = context.vm.read::<u8>().into();
        let is_anonyms_function = context.vm.read::<u8>() != 0;
        Self::operation(
            class,
            name,
            function,
            is_anonyms_function,
            registers,
            context,
        )
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let class = context.vm.read::<u16>().into();
        let name = context.vm.read::<u16>().into();
        let function = context.vm.read::<u16>().into();
        let is_anonyms_function = context.vm.read::<u8>() != 0;
        Self::operation(
            class,
            name,
            function,
            is_anonyms_function,
            registers,
            context,
        )
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let class = context.vm.read::<u32>();
        let name = context.vm.read::<u32>();
        let function = context.vm.read::<u32>();
        let is_anonyms_function = context.vm.read::<u8>() != 0;
        Self::operation(
            class,
            name,
            function,
            is_anonyms_function,
            registers,
            context,
        )
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
    fn operation(
        class: u32,
        function: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let class = registers.get(class);
        let function = registers.get(function);
        let name = context.vm.frame().code_block().constant_string(index);

        let function = function
            .as_object()
            .expect("field value must be function object");
        let class = class.as_object().expect("class must be function object");

        function
            .downcast_mut::<OrdinaryFunction>()
            .expect("field value must be function object")
            .set_home_object(class.clone());

        class
            .downcast_mut::<OrdinaryFunction>()
            .expect("class must be function object")
            .push_field_private(
                class.private_name(name),
                JsFunction::from_object_unchecked(function.clone()),
            );
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushClassFieldPrivate {
    const NAME: &'static str = "PushClassFieldPrivate";
    const INSTRUCTION: &'static str = "INST - PushClassFieldPrivate";
    const COST: u8 = 3;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let class = context.vm.read::<u8>().into();
        let function = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(class, function, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let class = context.vm.read::<u16>().into();
        let function = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(class, function, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let class = context.vm.read::<u32>();
        let function = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(class, function, index, registers, context)
    }
}
