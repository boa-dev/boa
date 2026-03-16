use crate::{Context, vm::opcode::Operation};

/// `PushDisposalScope` marks the current disposal stack depth for a new scope.
///
/// This opcode is emitted at the beginning of blocks that contain `using` declarations.
///
/// Operation:
///  - Stack: **=>**
pub(crate) struct PushDisposalScope;

impl PushDisposalScope {
    pub(crate) fn operation((): (), context: &mut Context) {
        context.vm.push_disposal_scope();
    }
}

impl Operation for PushDisposalScope {
    const NAME: &'static str = "PushDisposalScope";
    const INSTRUCTION: &'static str = "INST - PushDisposalScope";
    const COST: u8 = 1;
}
