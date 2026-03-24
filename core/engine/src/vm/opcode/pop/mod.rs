use crate::{Context, vm::opcode::Operation};

/// `Pop` implements the Opcode Operation for `Opcode::Pop`
///
/// Operation:
///  - Pop the top value from the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Pop;

impl Pop {
    #[inline(always)]
    pub(super) fn operation((): (), context: &mut Context) {
        let _val = context.vm.stack.pop();
    }
}

impl Operation for Pop {
    const NAME: &'static str = "Pop";
    const INSTRUCTION: &'static str = "INST - Pop";
    const COST: u8 = 1;
}

/// `PopEnvironment` implements the Opcode Operation for `Opcode::PopEnvironment`
///
/// Operation:
///  - Pop the current environment.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PopEnvironment;

impl PopEnvironment {
    #[inline(always)]
    pub(super) fn operation((): (), context: &mut Context) {
        // Pop and dispose the resource stack for the exiting lexical scope.
        // This must happen BEFORE the environment is popped, so that
        // dispose methods can still access bindings in the current scope.
        //
        // See: https://tc39.es/proposal-explicit-resource-management/#sec-disposeresources
        let popped = context.vm.frame_mut().disposable_resource_stacks.pop();
        if let Some(mut stack) = popped {
            crate::resource_management::dispose_resources(context, &mut stack);
        }

        context.vm.frame_mut().environments.pop();
    }
}

impl Operation for PopEnvironment {
    const NAME: &'static str = "PopEnvironment";
    const INSTRUCTION: &'static str = "INST - PopEnvironment";
    const COST: u8 = 1;
}
