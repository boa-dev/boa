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

/// `DefineClassStaticGetterByName` implements the Opcode Operation for `Opcode::DefineClassStaticGetterByName`
///
/// Operation:
///  - Defines a class getter by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticGetterByName;

impl DefineClassStaticGetterByName {
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
            set_function_name(function_object, &key, Some(js_str!("get")), context);
            function_object
                .downcast_mut::<OrdinaryFunction>()
                .expect("method must be function object")
                .set_home_object(class.clone());
        }
        let set = class
            .__get_own_property__(&key, &mut InternalMethodContext::new(context))?
            .as_ref()
            .and_then(PropertyDescriptor::set)
            .cloned();
        class.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .maybe_get(Some(function.clone()))
                .maybe_set(set)
                .enumerable(false)
                .configurable(true)
                .build(),
            &mut InternalMethodContext::new(context),
        )?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineClassStaticGetterByName {
    const NAME: &'static str = "DefineClassStaticGetterByName";
    const INSTRUCTION: &'static str = "INST - DefineClassStaticGetterByName";
    const COST: u8 = 6;
}

/// `DefineClassGetterByName` implements the Opcode Operation for `Opcode::DefineClassGetterByName`
///
/// Operation:
///  - Defines a class getter by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassGetterByName;

impl DefineClassGetterByName {
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
            set_function_name(function_object, &key, Some(js_str!("get")), context);
            function_object
                .downcast_mut::<OrdinaryFunction>()
                .expect("method must be function object")
                .set_home_object(class_proto.clone());
        }
        let set = class_proto
            .__get_own_property__(&key, &mut InternalMethodContext::new(context))?
            .as_ref()
            .and_then(PropertyDescriptor::set)
            .cloned();
        class_proto.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .maybe_get(Some(function.clone()))
                .maybe_set(set)
                .enumerable(false)
                .configurable(true)
                .build(),
            &mut InternalMethodContext::new(context),
        )?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineClassGetterByName {
    const NAME: &'static str = "DefineClassGetterByName";
    const INSTRUCTION: &'static str = "INST - DefineClassGetterByName";
    const COST: u8 = 6;
}

/// `DefineClassStaticGetterByValue` implements the Opcode Operation for `Opcode::DefineClassStaticGetterByValue`
///
/// Operation:
///  - Defines a class getter by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticGetterByValue;

impl DefineClassStaticGetterByValue {
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
            set_function_name(function_object, &key, Some(js_str!("get")), context);
            function_object
                .downcast_mut::<OrdinaryFunction>()
                .expect("method must be function object")
                .set_home_object(class.clone());
        }

        let set = class
            .__get_own_property__(&key, &mut InternalMethodContext::new(context))?
            .as_ref()
            .and_then(PropertyDescriptor::set)
            .cloned();
        class.define_property_or_throw(
            key,
            PropertyDescriptor::builder()
                .maybe_get(Some(function.clone()))
                .maybe_set(set)
                .enumerable(false)
                .configurable(true)
                .build(),
            context,
        )?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineClassStaticGetterByValue {
    const NAME: &'static str = "DefineClassStaticGetterByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassStaticGetterByValue";
    const COST: u8 = 6;
}

/// `DefineClassGetterByValue` implements the Opcode Operation for `Opcode::DefineClassGetterByValue`
///
/// Operation:
///  - Defines a class getter by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassGetterByValue;

impl DefineClassGetterByValue {
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
            set_function_name(function_object, &key, Some(js_str!("get")), context);
            function_object
                .downcast_mut::<OrdinaryFunction>()
                .expect("method must be function object")
                .set_home_object(class_proto.clone());
        }
        let set = class_proto
            .__get_own_property__(&key, &mut InternalMethodContext::new(context))?
            .as_ref()
            .and_then(PropertyDescriptor::set)
            .cloned();
        class_proto.__define_own_property__(
            &key,
            PropertyDescriptor::builder()
                .maybe_get(Some(function.clone()))
                .maybe_set(set)
                .enumerable(false)
                .configurable(true)
                .build(),
            &mut InternalMethodContext::new(context),
        )?;
        Ok(CompletionType::Normal)
    }
}

impl Operation for DefineClassGetterByValue {
    const NAME: &'static str = "DefineClassGetterByValue";
    const INSTRUCTION: &'static str = "INST - DefineClassGetterByValue";
    const COST: u8 = 6;
}
