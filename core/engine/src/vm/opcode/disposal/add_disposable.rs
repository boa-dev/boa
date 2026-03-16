use crate::{
    Context, JsResult,
    vm::opcode::{Operation, RegisterOperand},
};

/// `AddDisposableResource` implements the AddDisposableResource operation.
///
/// This opcode adds a resource to the disposal stack for later cleanup.
///
/// Operation:
///  - Stack: **=>**
///  - Registers:
///    - Input: value
pub(crate) struct AddDisposableResource;

impl AddDisposableResource {
    pub(crate) fn operation(value: RegisterOperand, context: &mut Context) -> JsResult<()> {
        let value = context.vm.get_register(value.into()).clone();

        // Per spec: If value is null or undefined, return
        if value.is_null_or_undefined() {
            return Ok(());
        }

        // Get the dispose method (value[Symbol.dispose])
        let key = crate::JsSymbol::dispose();
        let dispose_method = value.get_method(key, context)?;

        // If dispose method is None, return
        let Some(dispose_method) = dispose_method else {
            return Ok(());
        };

        // Add to disposal stack
        context
            .vm
            .push_disposable_resource(value, dispose_method.into());

        Ok(())
    }
}

impl Operation for AddDisposableResource {
    const NAME: &'static str = "AddDisposableResource";
    const INSTRUCTION: &'static str = "INST - AddDisposableResource";
    const COST: u8 = 3;
}
