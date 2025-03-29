use boa_macros::js_str;

use crate::{
    builtins::function::{set_function_name, OrdinaryFunction},
    object::internal_methods::InternalMethodContext,
    property::PropertyDescriptor,
    vm::{
        opcode::{Operation, VaryingOperand},
        CompletionType, Registers,
    },
    Context, JsResult,
};

/// `DefineClassStaticSetterByName` implements the Opcode Operation for `Opcode::DefineClassStaticSetterByName`
///
/// Operation:
///  - Defines a class setter by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticSetterByName;

impl DefineClassStaticSetterByName {
    #[inline(always)]
    pub(crate) fn operation(
        (function, class, index): (VaryingOperand, VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let function = registers.get(function.into());
        let class = registers.get(class.into());
        let class = class.as_object().expect("class must be object");
        let key = context
            .vm
            .frame()
            .code_block()
            .constant_string(index.into())
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
}

/// `DefineClassSetterByName` implements the Opcode Operation for `Opcode::DefineClassSetterByName`
///
/// Operation:
///  - Defines a class setter by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassSetterByName;

impl DefineClassSetterByName {
    #[inline(always)]
    pub(crate) fn operation(
        (function, class_proto, index): (VaryingOperand, VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let function = registers.get(function.into());
        let class_proto = registers.get(class_proto.into());
        let class_proto = class_proto.as_object().expect("class must be object");
        let key = context
            .vm
            .frame()
            .code_block()
            .constant_string(index.into())
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
}

/// `DefineClassStaticSetterByValue` implements the Opcode Operation for `Opcode::DefineClassStaticSetterByValue`
///
/// Operation:
///  - Defines a class setter by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticSetterByValue;

impl DefineClassStaticSetterByValue {
    #[inline(always)]
    pub(crate) fn operation(
        (function, key, class): (VaryingOperand, VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let function = registers.get(function.into());
        let key = registers.get(key.into());
        let class = registers.get(class.into());
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
}

/// `DefineClassSetterByValue` implements the Opcode Operation for `Opcode::DefineClassSetterByValue`
///
/// Operation:
///  - Defines a class setter by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassSetterByValue;

impl DefineClassSetterByValue {
    #[inline(always)]
    pub(crate) fn operation(
        (function, key, class_proto): (VaryingOperand, VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let function = registers.get(function.into());
        let key = registers.get(key.into());
        let class_proto = registers.get(class_proto.into());
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
}
