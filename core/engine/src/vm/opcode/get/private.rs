use crate::{
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

/// `GetPrivateField` implements the Opcode Operation for `Opcode::GetPrivateField`
///
/// Operation:
///  - Get a private property by name from an object an push it on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetPrivateField;

impl GetPrivateField {
    fn operation(
        dst: u32,
        object: u32,
        index: usize,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let name = context.vm.frame().code_block().constant_string(index);
        let object = registers.get(object);
        let object = object.to_object(context)?;
        let name = context
            .vm
            .environments
            .resolve_private_identifier(name)
            .expect("private name must be in environment");

        let result = object.private_get(&name, context)?;
        registers.set(dst, result);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetPrivateField {
    const NAME: &'static str = "GetPrivateField";
    const INSTRUCTION: &'static str = "INST - GetPrivateField";
    const COST: u8 = 4;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u8>().into();
        let object = context.vm.read::<u8>().into();
        let index = context.vm.read::<u8>() as usize;
        Self::operation(dst, object, index, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u16>().into();
        let object = context.vm.read::<u16>().into();
        let index = context.vm.read::<u16>() as usize;
        Self::operation(dst, object, index, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u32>();
        let object = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        Self::operation(dst, object, index, registers, context)
    }
}
