use crate::{
    Context,
    builtins::function::OrdinaryFunction,
    js_str, js_string,
    object::{PrivateElement, internal_methods::InternalMethodPropertyContext},
    property::PropertyDescriptor,
    vm::opcode::{Operation, VaryingOperand},
};

/// `PushClassPrivateMethod` implements the Opcode Operation for `Opcode::PushClassPrivateMethod`
///
/// Operation:
///  - Push a private method to the class.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushClassPrivateMethod;

impl PushClassPrivateMethod {
    #[inline(always)]
    pub(crate) fn operation(
        (object, prototype, value, index): (
            VaryingOperand,
            VaryingOperand,
            VaryingOperand,
            VaryingOperand,
        ),
        context: &Context,
    ) {
        let object = context.vm_mut().get_register(object.into()).clone();
        let prototype = context.vm_mut().get_register(prototype.into()).clone();
        let value = context.vm_mut().get_register(value.into()).clone();
        let name = context
            .vm_mut()
            .frame()
            .code_block()
            .constant_string(index.into());

        let value = value.as_callable().expect("method must be callable");
        let prototype = prototype
            .as_object()
            .expect("class_prototype must be function object");
        let object = object.as_object().expect("class must be function object");

        let name_string = js_string!(js_str!("#"), &name);
        let desc = PropertyDescriptor::builder()
            .value(name_string)
            .writable(false)
            .enumerable(false)
            .configurable(true)
            .build();
        value
            .__define_own_property__(
                &js_string!("name").into(),
                desc,
                &mut InternalMethodPropertyContext::new(context),
            )
            .expect("failed to set name property on private method");
        value
            .downcast_mut::<OrdinaryFunction>()
            .expect("method must be function object")
            .set_home_object(prototype.clone());

        object
            .downcast_mut::<OrdinaryFunction>()
            .expect("class must be function object")
            .push_private_method(
                object.private_name(name),
                PrivateElement::Method(value.clone()),
            );
    }
}

impl Operation for PushClassPrivateMethod {
    const NAME: &'static str = "PushClassPrivateMethod";
    const INSTRUCTION: &'static str = "INST - PushClassPrivateMethod";
    const COST: u8 = 6;
}

/// `PushClassPrivateGetter` implements the Opcode Operation for `Opcode::PushClassPrivateGetter`
///
/// Operation:
///  - Push a private getter to the class.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushClassPrivateGetter;

impl PushClassPrivateGetter {
    #[inline(always)]
    pub(crate) fn operation(
        (object, value, index): (VaryingOperand, VaryingOperand, VaryingOperand),
        context: &Context,
    ) {
        let vm = context.vm_mut();
        let object = vm.get_register(object.into()).clone();
        let value = vm.get_register(value.into()).clone();
        let name = vm.frame().code_block().constant_string(index.into());

        let value = value.as_callable().expect("getter must be callable");
        let object = object.as_object().expect("class must be function object");

        object
            .downcast_mut::<OrdinaryFunction>()
            .expect("class must be function object")
            .push_private_method(
                object.private_name(name),
                PrivateElement::Accessor {
                    getter: Some(value),
                    setter: None,
                },
            );
    }
}

impl Operation for PushClassPrivateGetter {
    const NAME: &'static str = "PushClassPrivateGetter";
    const INSTRUCTION: &'static str = "INST - PushClassPrivateGetter";
    const COST: u8 = 6;
}

/// `PushClassPrivateSetter` implements the Opcode Operation for `Opcode::PushClassPrivateSetter`
///
/// Operation:
///  - Push a private setter to the class.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushClassPrivateSetter;

impl PushClassPrivateSetter {
    #[inline(always)]
    pub(crate) fn operation(
        (object, value, index): (VaryingOperand, VaryingOperand, VaryingOperand),
        context: &Context,
    ) {
        let vm = context.vm_mut();
        let object = vm.get_register(object.into()).clone();
        let value = vm.get_register(value.into()).clone();
        let name = vm.frame().code_block().constant_string(index.into());

        let value = value.as_callable().expect("getter must be callable");
        let object = object.as_object().expect("class must be function object");

        object
            .downcast_mut::<OrdinaryFunction>()
            .expect("class must be function object")
            .push_private_method(
                object.private_name(name),
                PrivateElement::Accessor {
                    getter: None,
                    setter: Some(value),
                },
            );
    }
}

impl Operation for PushClassPrivateSetter {
    const NAME: &'static str = "PushClassPrivateSetter";
    const INSTRUCTION: &'static str = "INST - PushClassPrivateSetter";
    const COST: u8 = 6;
}
