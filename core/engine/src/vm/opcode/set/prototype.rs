use crate::{
    object::internal_methods::InternalMethodContext,
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

/// `SetPrototype` implements the Opcode Operation for `Opcode::SetPrototype`
///
/// Operation:
///  - Sets the prototype of an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrototype;

impl SetPrototype {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        object: u32,
        value: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let object = registers.get(object);
        let value = registers.get(value);

        let prototype = if let Some(prototype) = value.as_object() {
            Some(prototype.clone())
        } else if value.is_null() {
            None
        } else {
            return Ok(CompletionType::Normal);
        };

        let object = object.as_object().expect("object is not an object");
        object
            .__set_prototype_of__(prototype, &mut InternalMethodContext::new(context))
            .expect("cannot fail per spec");

        Ok(CompletionType::Normal)
    }
}

impl Operation for SetPrototype {
    const NAME: &'static str = "SetPrototype";
    const INSTRUCTION: &'static str = "INST - SetPrototype";
    const COST: u8 = 4;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u8>().into();
        let value = context.vm.read::<u8>().into();
        Self::operation(object, value, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u16>().into();
        let value = context.vm.read::<u16>().into();
        Self::operation(object, value, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let object = context.vm.read::<u32>();
        let value = context.vm.read::<u32>();
        Self::operation(object, value, registers, context)
    }
}
