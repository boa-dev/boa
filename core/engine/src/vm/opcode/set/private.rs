use crate::{
    js_str, js_string,
    object::PrivateElement,
    property::PropertyDescriptor,
    vm::opcode::{Operation, VaryingOperand},
    Context, JsResult,
};

/// `SetPrivateField` implements the Opcode Operation for `Opcode::SetPrivateField`
///
/// Operation:
///  - Assign the value of a private property of an object by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrivateField;

impl SetPrivateField {
    #[inline(always)]
    pub(crate) fn operation(
        (value, object, index): (VaryingOperand, VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let name = context
            .vm
            .frame()
            .code_block()
            .constant_string(index.into());
        let value = context.vm.get_register(value.into()).clone();
        let object = context.vm.get_register(object.into()).clone();
        let base_obj = object.to_object(context)?;
        let name = context
            .vm
            .environments
            .resolve_private_identifier(name)
            .expect("private name must be in environment");

        base_obj.private_set(&name, value.clone(), context)?;
        Ok(())
    }
}

impl Operation for SetPrivateField {
    const NAME: &'static str = "SetPrivateField";
    const INSTRUCTION: &'static str = "INST - SetPrivateField";
    const COST: u8 = 4;
}

/// `DefinePrivateField` implements the Opcode Operation for `Opcode::DefinePrivateField`
///
/// Operation:
///  - Set a private property of a class constructor by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DefinePrivateField;

impl DefinePrivateField {
    #[inline(always)]
    pub(crate) fn operation(
        (object, value, index): (VaryingOperand, VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) {
        let object = context.vm.get_register(object.into());
        let value = context.vm.get_register(value.into());
        let name = context
            .vm
            .frame()
            .code_block()
            .constant_string(index.into());

        let object = object
            .as_object()
            .expect("class prototype must be an object");

        object.borrow_mut().append_private_element(
            object.private_name(name),
            PrivateElement::Field(value.clone()),
        );
    }
}

impl Operation for DefinePrivateField {
    const NAME: &'static str = "DefinePrivateField";
    const INSTRUCTION: &'static str = "INST - DefinePrivateField";
    const COST: u8 = 4;
}

/// `SetPrivateMethod` implements the Opcode Operation for `Opcode::SetPrivateMethod`
///
/// Operation:
///  - Set a private method of a class constructor by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrivateMethod;

impl SetPrivateMethod {
    #[inline(always)]
    pub(crate) fn operation(
        (object, value, index): (VaryingOperand, VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) {
        let object = context.vm.get_register(object.into()).clone();
        let value = context.vm.get_register(value.into()).clone();
        let name = context
            .vm
            .frame()
            .code_block()
            .constant_string(index.into());

        let value = value.as_callable().expect("method must be callable");
        let object = object
            .as_object()
            .expect("class prototype must be an object");

        let name_string = js_string!(js_str!("#"), &name);
        let desc = PropertyDescriptor::builder()
            .value(name_string)
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build();
        value
            .__define_own_property__(&js_string!("name").into(), desc, &mut context.into())
            .expect("failed to set name property on private method");

        object.borrow_mut().append_private_element(
            object.private_name(name),
            PrivateElement::Method(value.clone()),
        );
    }
}

impl Operation for SetPrivateMethod {
    const NAME: &'static str = "SetPrivateMethod";
    const INSTRUCTION: &'static str = "INST - SetPrivateMethod";
    const COST: u8 = 4;
}

/// `SetPrivateSetter` implements the Opcode Operation for `Opcode::SetPrivateSetter`
///
/// Operation:
///  - Set a private setter property of a class constructor by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrivateSetter;

impl SetPrivateSetter {
    #[inline(always)]
    pub(crate) fn operation(
        (object, value, index): (VaryingOperand, VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) {
        let object = context.vm.get_register(object.into());
        let value = context.vm.get_register(value.into());
        let name = context
            .vm
            .frame()
            .code_block()
            .constant_string(index.into());

        let value = value.as_callable().expect("setter must be callable");
        let object = object
            .as_object()
            .expect("class prototype must be an object");

        object.borrow_mut().append_private_element(
            object.private_name(name),
            PrivateElement::Accessor {
                getter: None,
                setter: Some(value.clone()),
            },
        );
    }
}

impl Operation for SetPrivateSetter {
    const NAME: &'static str = "SetPrivateSetter";
    const INSTRUCTION: &'static str = "INST - SetPrivateSetter";
    const COST: u8 = 4;
}

/// `SetPrivateGetter` implements the Opcode Operation for `Opcode::SetPrivateGetter`
///
/// Operation:
///  - Set a private getter property of a class constructor by it's name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrivateGetter;

impl SetPrivateGetter {
    #[inline(always)]
    pub(crate) fn operation(
        (object, value, index): (VaryingOperand, VaryingOperand, VaryingOperand),
        context: &mut Context,
    ) {
        let object = context.vm.get_register(object.into());
        let value = context.vm.get_register(value.into());
        let name = context
            .vm
            .frame()
            .code_block()
            .constant_string(index.into());

        let value = value.as_callable().expect("getter must be callable");
        let object = object
            .as_object()
            .expect("class prototype must be an object");

        object.borrow_mut().append_private_element(
            object.private_name(name),
            PrivateElement::Accessor {
                getter: Some(value.clone()),
                setter: None,
            },
        );
    }
}

impl Operation for SetPrivateGetter {
    const NAME: &'static str = "SetPrivateGetter";
    const INSTRUCTION: &'static str = "INST - SetPrivateGetter";
    const COST: u8 = 4;
}
