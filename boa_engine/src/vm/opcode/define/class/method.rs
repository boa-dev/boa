use crate::{
    builtins::function::set_function_name,
    property::PropertyDescriptor,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `DefineClassStaticMethodByName` implements the Opcode Operation for `Opcode::DefineClassStaticMethodByName`
///
/// Operation:
///  - Defines a class method by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticMethodByName;

impl DefineClassStaticMethodByName {
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let function = context.vm.pop();
        let class = context.vm.pop();
        let class = class.as_object().expect("class must be object");
        let key = context.vm.frame().code_block.names[index].clone().into();
        {
            let function_object = function
                .as_object()
                .expect("method must be function object");
            set_function_name(function_object, &key, None, context);
            let mut function_object = function_object.borrow_mut();
            let function_mut = function_object
                .as_function_mut()
                .expect("method must be function object");
            function_mut.set_home_object(class.clone());
        }

        class.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineClassStaticMethodByName {
    const NAME: &'static str = "DefineClassStaticMethodByName";
    const INSTRUCTION: &'static str = "INST - DefineClassStaticMethodByName";

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

/// `DefineClassMethodByName` implements the Opcode Operation for `Opcode::DefineClassMethodByName`
///
/// Operation:
///  - Defines a class method by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassMethodByName;

impl DefineClassMethodByName {
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let function = context.vm.pop();
        let class_proto = context.vm.pop();
        let class_proto = class_proto.as_object().expect("class must be object");
        let key = context.vm.frame().code_block.names[index].clone().into();
        {
            let function_object = function
                .as_object()
                .expect("method must be function object");
            set_function_name(function_object, &key, None, context);
            let mut function_object = function_object.borrow_mut();
            let function_mut = function_object
                .as_function_mut()
                .expect("method must be function object");
            function_mut.set_home_object(class_proto.clone());
        }

        class_proto.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineClassMethodByName {
    const NAME: &'static str = "DefineClassMethodByName";
    const INSTRUCTION: &'static str = "INST - DefineClassMethodByName";

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

/// `DefineClassStaticMethodByValue` implements the Opcode Operation for `Opcode::DefineClassStaticMethodByValue`
///
/// Operation:
///  - Defines a class method by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticMethodByValue;

impl Operation for DefineClassStaticMethodByValue {
    const NAME: &'static str = "DefineClassStaticMethodByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassStaticMethodByValue";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let function = context.vm.pop();
        let key = context.vm.pop();
        let class = context.vm.pop();
        let class = class.as_object().expect("class must be object");
        let key = key
            .to_property_key(context)
            .expect("property key must already be valid");
        {
            let function_object = function
                .as_object()
                .expect("method must be function object");
            set_function_name(function_object, &key, None, context);
            let mut function_object_mut = function_object.borrow_mut();
            let function_mut = function_object_mut
                .as_function_mut()
                .expect("method must be function object");
            function_mut.set_home_object(class.clone());
        }

        class.define_property_or_throw(
            key,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(CompletionType::Normal)
    }
}

/// `DefineClassMethodByValue` implements the Opcode Operation for `Opcode::DefineClassMethodByValue`
///
/// Operation:
///  - Defines a class method by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassMethodByValue;

impl Operation for DefineClassMethodByValue {
    const NAME: &'static str = "DefineClassMethodByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassMethodByValue";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let function = context.vm.pop();
        let key = context.vm.pop();
        let class_proto = context.vm.pop();
        let class_proto = class_proto.as_object().expect("class must be object");
        let key = key
            .to_property_key(context)
            .expect("property key must already be valid");
        {
            let function_object = function
                .as_object()
                .expect("method must be function object");
            set_function_name(function_object, &key, None, context);
            let mut function_object = function_object.borrow_mut();
            let function_mut = function_object
                .as_function_mut()
                .expect("method must be function object");
            function_mut.set_home_object(class_proto.clone());
        }

        class_proto.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .value(function)
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(CompletionType::Normal)
    }
}
