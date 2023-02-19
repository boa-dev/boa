use crate::{
    vm::{ok_or_throw_completion, opcode::Operation, CompletionType},
    Context,
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

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let index = context.vm.read::<u32>();
        let name = context.vm.frame().code_block.private_names[index as usize];
        let value = context.vm.pop();
        let base_obj = ok_or_throw_completion!(value.to_object(context), context);
        let result = ok_or_throw_completion!(base_obj.private_get(&name, context), context);
        context.vm.push(result);
        CompletionType::Normal
    }
}
