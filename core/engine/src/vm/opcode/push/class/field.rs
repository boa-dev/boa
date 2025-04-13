use crate::{
    builtins::function::OrdinaryFunction,
    object::JsFunction,
    vm::{
        opcode::{Operation, VaryingOperand},
        Registers,
    },
    Context, JsResult,
};

/// `PushClassField` implements the Opcode Operation for `Opcode::PushClassField`
///
/// Operation:
///  - Push a field to a class.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushClassField;

impl PushClassField {
    #[inline(always)]
    pub(crate) fn operation(
        (class, name, function, is_anonyms_function): (
            VaryingOperand,
            VaryingOperand,
            VaryingOperand,
            VaryingOperand,
        ),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        let class = registers.get(class.into());
        let name = registers.get(name.into());
        let function = registers.get(function.into());
        let is_anonyms_function = u32::from(is_anonyms_function) != 0;

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
        Ok(())
    }
}

impl Operation for PushClassField {
    const NAME: &'static str = "PushClassField";
    const INSTRUCTION: &'static str = "INST - PushClassField";
    const COST: u8 = 6;
}

/// `PushClassFieldPrivate` implements the Opcode Operation for `Opcode::PushClassFieldPrivate`
///
/// Operation:
///  - Push a private field to the class.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushClassFieldPrivate;

impl PushClassFieldPrivate {
    #[inline(always)]
    pub(crate) fn operation(
        (class, function, index): (VaryingOperand, VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) {
        let class = registers.get(class.into());
        let function = registers.get(function.into());
        let name = context
            .vm
            .frame()
            .code_block()
            .constant_string(index.into());

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
    }
}

impl Operation for PushClassFieldPrivate {
    const NAME: &'static str = "PushClassFieldPrivate";
    const INSTRUCTION: &'static str = "INST - PushClassFieldPrivate";
    const COST: u8 = 3;
}
