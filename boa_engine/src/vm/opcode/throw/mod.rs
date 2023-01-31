use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsError, JsResult,
};

/// `Throw` implements the Opcode Operation for `Opcode::Throw`
///
/// Operation:
///  - Throw exception.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Throw;

impl Operation for Throw {
    const NAME: &'static str = "Throw";
    const INSTRUCTION: &'static str = "INST - Throw";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let err = context.vm.pop();
        Err(JsError::from_opaque(err))
    }
}
