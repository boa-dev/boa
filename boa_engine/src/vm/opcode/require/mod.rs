use crate::{
    vm::{ok_or_throw_completion, opcode::Operation, CompletionType},
    Context,
};

/// `RequireObjectCoercible` implements the Opcode Operation for `Opcode::RequireObjectCoercible`
///
/// Operation:
///  - Call `RequireObjectCoercible` on the stack value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RequireObjectCoercible;

impl Operation for RequireObjectCoercible {
    const NAME: &'static str = "RequireObjectCoercible";
    const INSTRUCTION: &'static str = "INST - RequireObjectCoercible";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        let value = ok_or_throw_completion!(value.require_object_coercible(), context);
        context.vm.push(value.clone());
        CompletionType::Normal
    }
}
