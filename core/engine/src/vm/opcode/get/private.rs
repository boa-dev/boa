use crate::{
    Context, JsResult,
    vm::opcode::{Operation, VaryingOperand},
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
        context: &Context,
    ) -> JsResult<()> {
        let name = context.with_vm(|vm| vm.frame().code_block().constant_string(index.into()));
        let object = context.get_register(object.into());
        let object = object.to_object(context)?;
        let name = context
            .with_vm(|vm| vm.frame.environments.resolve_private_identifier(name))
            .expect("private name must be in environment");

        let result = object.private_get(&name, context)?;
        context.set_register(dst.into(), result);
        Ok(())
    }
}

impl Operation for GetPrivateField {
    const NAME: &'static str = "GetPrivateField";
    const INSTRUCTION: &'static str = "INST - GetPrivateField";
    const COST: u8 = 4;
}
