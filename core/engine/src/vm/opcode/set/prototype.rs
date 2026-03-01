use crate::{
    Context,
    object::internal_methods::InternalMethodPropertyContext,
    vm::opcode::{Operation, VaryingOperand},
};

/// `SetPrototype` implements the Opcode Operation for `Opcode::SetPrototype`
///
/// Operation:
///  - Sets the prototype of an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrototype;

impl SetPrototype {
    #[inline(always)]
    pub(crate) fn operation((object, value): (VaryingOperand, VaryingOperand), context: &Context) {
        let vm = context.vm_mut();
        let object = vm.get_register(object.into()).clone();
        let value = vm.get_register(value.into()).clone();

        let prototype = if let Some(prototype) = value.as_object() {
            Some(prototype)
        } else if value.is_null() {
            None
        } else {
            return;
        };

        let object = object.as_object().expect("object is not an object");
        object
            .__set_prototype_of__(prototype, &InternalMethodPropertyContext::new(context))
            .expect("cannot fail per spec");
    }
}

impl Operation for SetPrototype {
    const NAME: &'static str = "SetPrototype";
    const INSTRUCTION: &'static str = "INST - SetPrototype";
    const COST: u8 = 4;
}
