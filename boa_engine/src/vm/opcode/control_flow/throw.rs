use crate::{
    vm::{opcode::Operation, CompletionType},
    Context,
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

    fn execute(_: &mut Context<'_>) -> CompletionType {
        CompletionType::Throw
    }
}
