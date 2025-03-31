use crate::{
    vm::{
        opcode::{Operation, VaryingOperand},
        Registers,
    },
    Context, JsResult,
};

/// `GetPrivateField` implements the Opcode Operation for `Opcode::GetPrivateField`
///
/// Operation:
///  - Get a private property by name from an object an push it on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetPrivateField;

impl GetPrivateField {
    #[inline(always)]
    pub(crate) fn operation(
        (dst, object, index): (VaryingOperand, VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        let name = context
            .vm
            .frame()
            .code_block()
            .constant_string(index.into());
        let object = registers.get(object.into());
        let object = object.to_object(context)?;
        let name = context
            .vm
            .environments
            .resolve_private_identifier(name)
            .expect("private name must be in environment");

        let result = object.private_get(&name, context)?;
        registers.set(dst.into(), result);
        Ok(())
    }
}

impl Operation for GetPrivateField {
    const NAME: &'static str = "GetPrivateField";
    const INSTRUCTION: &'static str = "INST - GetPrivateField";
    const COST: u8 = 4;
}
