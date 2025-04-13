use crate::{
    object::internal_methods::InternalMethodContext,
    vm::{
        opcode::{Operation, VaryingOperand},
        Registers,
    },
    Context,
};

/// `SetPrototype` implements the Opcode Operation for `Opcode::SetPrototype`
///
/// Operation:
///  - Sets the prototype of an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrototype;

impl SetPrototype {
    #[inline(always)]
    pub(crate) fn operation(
        (object, value): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) {
        let object = registers.get(object.into());
        let value = registers.get(value.into());

        let prototype = if let Some(prototype) = value.as_object() {
            Some(prototype.clone())
        } else if value.is_null() {
            None
        } else {
            return;
        };

        let object = object.as_object().expect("object is not an object");
        object
            .__set_prototype_of__(prototype, &mut InternalMethodContext::new(context))
            .expect("cannot fail per spec");
    }
}

impl Operation for SetPrototype {
    const NAME: &'static str = "SetPrototype";
    const INSTRUCTION: &'static str = "INST - SetPrototype";
    const COST: u8 = 4;
}
