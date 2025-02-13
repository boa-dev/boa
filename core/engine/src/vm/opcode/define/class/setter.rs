use boa_macros::js_str;

use crate::{
    builtins::function::{set_function_name, OrdinaryFunction},
    object::internal_methods::InternalMethodContext,
    property::PropertyDescriptor,
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

/// `DefineClassStaticSetterByName` implements the Opcode Operation for `Opcode::DefineClassStaticSetterByName`
///
/// Operation:
///  - Defines a class setter by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticSetterByName;

impl DefineClassStaticSetterByName {
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
            set_function_name(function_object, &key, Some(js_str!("set")), context);
            function_object
                .downcast_mut::<OrdinaryFunction>()
                .expect("method must be function object")
                .set_home_object(class.clone());
        }
        let get = class
            .__get_own_property__(&key, &mut InternalMethodContext::new(context))?
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();

        class.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .maybe_set(Some(function.clone()))
                .maybe_get(get)
                .enumerable(false)
                .configurable(true)
                .build(),
            &mut InternalMethodContext::new(context),
        )?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineClassStaticSetterByName {
    const NAME: &'static str = "DefineClassStaticSetterByName";
    const INSTRUCTION: &'static str = "INST - DefineClassStaticSetterByName";
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

/// `DefineClassSetterByName` implements the Opcode Operation for `Opcode::DefineClassSetterByName`
///
/// Operation:
///  - Defines a class setter by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassSetterByName;

impl DefineClassSetterByName {
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
            set_function_name(function_object, &key, Some(js_str!("set")), context);
            function_object
                .downcast_mut::<OrdinaryFunction>()
                .expect("method must be function object")
                .set_home_object(class_proto.clone());
        }
        let get = class_proto
            .__get_own_property__(&key, &mut InternalMethodContext::new(context))?
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();

        class_proto.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .maybe_set(Some(function.clone()))
                .maybe_get(get)
                .enumerable(false)
                .configurable(true)
                .build(),
            &mut InternalMethodContext::new(context),
        )?;

        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineClassSetterByName {
    const NAME: &'static str = "DefineClassSetterByName";
    const INSTRUCTION: &'static str = "INST - DefineClassSetterByName";
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

/// `DefineClassStaticSetterByValue` implements the Opcode Operation for `Opcode::DefineClassStaticSetterByValue`
///
/// Operation:
///  - Defines a class setter by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticSetterByValue;

impl DefineClassStaticSetterByValue {
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
            set_function_name(function_object, &key, Some(js_str!("set")), context);
            function_object
                .downcast_mut::<OrdinaryFunction>()
                .expect("method must be function object")
                .set_home_object(class.clone());
        }
        let get = class
            .__get_own_property__(&key, &mut InternalMethodContext::new(context))?
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();

        class.define_property_or_throw(
            key,
            PropertyDescriptor::builder()
                .maybe_set(Some(function.clone()))
                .maybe_get(get)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;

        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineClassStaticSetterByValue {
    const NAME: &'static str = "DefineClassStaticSetterByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassStaticSetterByValue";
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

/// `DefineClassSetterByValue` implements the Opcode Operation for `Opcode::DefineClassSetterByValue`
///
/// Operation:
///  - Defines a class setter by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassSetterByValue;

impl DefineClassSetterByValue {
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
            set_function_name(function_object, &key, Some(js_str!("set")), context);
            function_object
                .downcast_mut::<OrdinaryFunction>()
                .expect("method must be function object")
                .set_home_object(class_proto.clone());
        }
        let get = class_proto
            .__get_own_property__(&key, &mut InternalMethodContext::new(context))?
            .as_ref()
            .and_then(PropertyDescriptor::get)
            .cloned();

        class_proto.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .maybe_set(Some(function.clone()))
                .maybe_get(get)
                .enumerable(false)
                .configurable(true)
                .build(),
            &mut InternalMethodContext::new(context),
        )?;

        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineClassSetterByValue {
    const NAME: &'static str = "DefineClassSetterByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassSetterByValue";
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
