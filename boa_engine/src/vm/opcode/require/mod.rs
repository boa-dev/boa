use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        let value = value.require_object_coercible()?;
        context.vm.push(value);
        Ok(ShouldExit::False)
    }
}
