use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `GetPrivateField` implements the Opcode Operation for `Opcode::GetPrivateField`
///
/// Operation:
///  - Get a private property by name from an object an push it on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetPrivateField;

impl Operation for GetPrivateField {
    const NAME: &'static str = "GetPrivateField";
    const INSTRUCTION: &'static str = "INST - GetPrivateField";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code_block.private_names[index as usize];
        let value = context.vm.pop();
        let base_obj = value.to_object(context)?;

        let name = context
            .vm
            .environments
            .resolve_private_identifier(name.description())
            .expect("private name must be in environment");

        let result = base_obj.private_get(&name, context)?;
        context.vm.push(result);
        Ok(CompletionType::Normal)
    }
}
