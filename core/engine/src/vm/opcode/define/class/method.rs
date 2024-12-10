use crate::{
    builtins::function::{set_function_name, OrdinaryFunction},
    object::internal_methods::InternalMethodContext,
    property::PropertyDescriptor,
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

/// `DefineClassStaticMethodByName` implements the Opcode Operation for `Opcode::DefineClassStaticMethodByName`
///
/// Operation:
///  - Defines a class method by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticMethodByName;

impl DefineClassStaticMethodByName {
    fn operation(
        class: u32,
        function: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let function = registers.get(function);
        let class = registers.get(class);
        let class = class.as_object().expect("class must be object");
        let key = context
            .vm
            .frame()
            .code_block()
            .constant_string(index)
            .into();
        {
            let function_object = function
                .as_object()
                .expect("method must be function object");
            set_function_name(function_object, &key, None, context);
            function_object
                .downcast_mut::<OrdinaryFunction>()
                .expect("method must be function object")
                .set_home_object(class.clone());
        }

        class.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .value(function.clone())
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
            &mut InternalMethodContext::new(context),
        )?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineClassStaticMethodByName {
    const NAME: &'static str = "DefineClassStaticMethodByName";
    const INSTRUCTION: &'static str = "INST - DefineClassStaticMethodByName";
    const COST: u8 = 6;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let function = context.vm.read::<u8>().into();
        let class = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(class, function, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let function = context.vm.read::<u16>().into();
        let class = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(class, function, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let function = context.vm.read::<u32>();
        let class = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(class, function, index, registers, context)
    }
}

/// `DefineClassMethodByName` implements the Opcode Operation for `Opcode::DefineClassMethodByName`
///
/// Operation:
///  - Defines a class method by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassMethodByName;

impl DefineClassMethodByName {
    fn operation(
        class_proto: u32,
        function: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let function = registers.get(function);
        let class_proto = registers.get(class_proto);
        let class_proto = class_proto.as_object().expect("class must be object");
        let key = context
            .vm
            .frame()
            .code_block()
            .constant_string(index)
            .into();
        {
            let function_object = function
                .as_object()
                .expect("method must be function object");
            set_function_name(function_object, &key, None, context);
            function_object
                .downcast_mut::<OrdinaryFunction>()
                .expect("method must be function object")
                .set_home_object(class_proto.clone());
        }

        class_proto.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .value(function.clone())
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
            &mut InternalMethodContext::new(context),
        )?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineClassMethodByName {
    const NAME: &'static str = "DefineClassMethodByName";
    const INSTRUCTION: &'static str = "INST - DefineClassMethodByName";
    const COST: u8 = 6;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let function = context.vm.read::<u8>().into();
        let class_proto = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(class_proto, function, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let function = context.vm.read::<u16>().into();
        let class_proto = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(class_proto, function, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let function = context.vm.read::<u32>();
        let class_proto = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(class_proto, function, index, registers, context)
    }
}

/// `DefineClassStaticMethodByValue` implements the Opcode Operation for `Opcode::DefineClassStaticMethodByValue`
///
/// Operation:
///  - Defines a class method by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticMethodByValue;

impl DefineClassStaticMethodByValue {
    fn operation(
        function: u32,
        key: u32,
        class: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let function = registers.get(function);
        let key = registers.get(key);
        let class = registers.get(class);
        let class = class.as_object().expect("class must be object");
        let key = key
            .to_property_key(context)
            .expect("property key must already be valid");
        {
            let function_object = function
                .as_object()
                .expect("method must be function object");
            set_function_name(function_object, &key, None, context);
            function_object
                .downcast_mut::<OrdinaryFunction>()
                .expect("method must be function object")
                .set_home_object(class.clone());
        }

        class.define_property_or_throw(
            key,
            PropertyDescriptor::builder()
                .value(function.clone())
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineClassStaticMethodByValue {
    const NAME: &'static str = "DefineClassStaticMethodByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassStaticMethodByValue";
    const COST: u8 = 6;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let function = context.vm.read::<u8>().into();
        let key = context.vm.read::<u8>().into();
        let class = context.vm.read::<u8>().into();
        Self::operation(function, key, class, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let function = context.vm.read::<u16>().into();
        let key = context.vm.read::<u16>().into();
        let class = context.vm.read::<u16>().into();
        Self::operation(function, key, class, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let function = context.vm.read::<u32>();
        let key = context.vm.read::<u32>();
        let class = context.vm.read::<u32>();
        Self::operation(function, key, class, registers, context)
    }
}

/// `DefineClassMethodByValue` implements the Opcode Operation for `Opcode::DefineClassMethodByValue`
///
/// Operation:
///  - Defines a class method by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassMethodByValue;

impl DefineClassMethodByValue {
    fn operation(
        function: u32,
        key: u32,
        class_proto: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let function = registers.get(function);
        let key = registers.get(key);
        let class_proto = registers.get(class_proto);
        let class_proto = class_proto.as_object().expect("class must be object");
        let key = key
            .to_property_key(context)
            .expect("property key must already be valid");
        {
            let function_object = function
                .as_object()
                .expect("method must be function object");
            set_function_name(function_object, &key, None, context);
            function_object
                .downcast_mut::<OrdinaryFunction>()
                .expect("method must be function object")
                .set_home_object(class_proto.clone());
        }

        class_proto.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .value(function.clone())
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
            &mut InternalMethodContext::new(context),
        )?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineClassMethodByValue {
    const NAME: &'static str = "DefineClassMethodByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassMethodByValue";
    const COST: u8 = 6;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let function = context.vm.read::<u8>().into();
        let key = context.vm.read::<u8>().into();
        let class_proto = context.vm.read::<u8>().into();
        Self::operation(function, key, class_proto, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let function = context.vm.read::<u16>().into();
        let key = context.vm.read::<u16>().into();
        let class_proto = context.vm.read::<u16>().into();
        Self::operation(function, key, class_proto, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let function = context.vm.read::<u32>();
        let key = context.vm.read::<u32>();
        let class_proto = context.vm.read::<u32>();
        Self::operation(function, key, class_proto, registers, context)
    }
}
