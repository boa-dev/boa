use crate::{
    builtins::function::set_function_name,
    property::PropertyDescriptor,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsString,
};

/// `DefineClassStaticSetterByName` implements the Opcode Operation for `Opcode::DefineClassStaticSetterByName`
///
/// Operation:
///  - Defines a class setter by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticSetterByName;

impl DefineClassStaticSetterByName {
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let function = context.vm.pop();
        let class = context.vm.pop();
        let class = class.as_object().expect("class must be object");
        let key = context.vm.frame().code_block.names[index].clone().into();
        {
            let function_object = function
                .as_object()
                .expect("method must be function object");
            set_function_name(function_object, &key, Some(JsString::from("set")), context);
            let mut function_object = function_object.borrow_mut();
            let function_mut = function_object
                .as_function_mut()
                .expect("method must be function object");
            function_mut.set_home_object(class.clone());
        }
        let get = class
            .__get_own_property__(&key, context)?
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();

        class.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .maybe_set(Some(function))
                .maybe_get(get)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineClassStaticSetterByName {
    const NAME: &'static str = "DefineClassStaticSetterByName";
    const INSTRUCTION: &'static str = "INST - DefineClassStaticSetterByName";

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

/// `DefineClassSetterByName` implements the Opcode Operation for `Opcode::DefineClassSetterByName`
///
/// Operation:
///  - Defines a class setter by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassSetterByName;

impl DefineClassSetterByName {
    fn operation(context: &mut Context<'_>, index: usize) -> JsResult<CompletionType> {
        let function = context.vm.pop();
        let class_proto = context.vm.pop();
        let class_proto = class_proto.as_object().expect("class must be object");
        let key = context.vm.frame().code_block.names[index].clone().into();
        {
            let function_object = function
                .as_object()
                .expect("method must be function object");
            set_function_name(function_object, &key, Some(JsString::from("set")), context);
            let mut function_object = function_object.borrow_mut();
            let function_mut = function_object
                .as_function_mut()
                .expect("method must be function object");
            function_mut.set_home_object(class_proto.clone());
        }
        let get = class_proto
            .__get_own_property__(&key, context)?
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();

        class_proto.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .maybe_set(Some(function))
                .maybe_get(get)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;

        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineClassSetterByName {
    const NAME: &'static str = "DefineClassSetterByName";
    const INSTRUCTION: &'static str = "INST - DefineClassSetterByName";

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

/// `DefineClassStaticSetterByValue` implements the Opcode Operation for `Opcode::DefineClassStaticSetterByValue`
///
/// Operation:
///  - Defines a class setter by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticSetterByValue;

impl Operation for DefineClassStaticSetterByValue {
    const NAME: &'static str = "DefineClassStaticSetterByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassStaticSetterByValue";

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
            set_function_name(function_object, &key, Some(JsString::from("set")), context);
            let mut function_object = function_object.borrow_mut();
            let function_mut = function_object
                .as_function_mut()
                .expect("method must be function object");
            function_mut.set_home_object(class.clone());
        }
        let get = class
            .__get_own_property__(&key, context)?
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();

        class.define_property_or_throw(
            key,
            PropertyDescriptor::builder()
                .maybe_set(Some(function))
                .maybe_get(get)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;

        Ok(CompletionType::Normal)
    }
}

/// `DefineClassSetterByValue` implements the Opcode Operation for `Opcode::DefineClassSetterByValue`
///
/// Operation:
///  - Defines a class setter by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassSetterByValue;

impl Operation for DefineClassSetterByValue {
    const NAME: &'static str = "DefineClassSetterByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassSetterByValue";

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
            set_function_name(function_object, &key, Some(JsString::from("set")), context);
            let mut function_object = function_object.borrow_mut();
            let function_mut = function_object
                .as_function_mut()
                .expect("method must be function object");
            function_mut.set_home_object(class_proto.clone());
        }
        let get = class_proto
            .__get_own_property__(&key, context)?
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();

        class_proto.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .maybe_set(Some(function))
                .maybe_get(get)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;

        Ok(CompletionType::Normal)
    }
}
