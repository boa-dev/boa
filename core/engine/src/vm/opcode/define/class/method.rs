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

/// `DefineClassStaticMethodByName` implements the Opcode Operation for `Opcode::DefineClassStaticMethodByName`
///
/// Operation:
///  - Defines a class method by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticMethodByName;

impl DefineClassStaticMethodByName {
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
}

/// `DefineClassMethodByName` implements the Opcode Operation for `Opcode::DefineClassMethodByName`
///
/// Operation:
///  - Defines a class method by name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassMethodByName;

impl DefineClassMethodByName {
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
}

/// `DefineClassStaticMethodByValue` implements the Opcode Operation for `Opcode::DefineClassStaticMethodByValue`
///
/// Operation:
///  - Defines a class method by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassStaticMethodByValue;

impl DefineClassStaticMethodByValue {
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
}

/// `DefineClassMethodByValue` implements the Opcode Operation for `Opcode::DefineClassMethodByValue`
///
/// Operation:
///  - Defines a class method by value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefineClassMethodByValue;

impl DefineClassMethodByValue {
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
}
