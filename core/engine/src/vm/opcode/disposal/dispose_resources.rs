use crate::{
    Context, JsError, JsNativeError, JsResult,
    vm::opcode::{IndexOperand, Operation},
};

/// `DisposeResources` implements the DisposeResources operation.
///
/// This opcode disposes the last `count` resources from the disposal stack.
/// The count is statically determined by the bytecompiler.
///
/// Operation:
///  - Stack: **=>**
pub(crate) struct DisposeResources;

impl DisposeResources {
    pub(crate) fn operation(count: IndexOperand, context: &mut Context) -> JsResult<()> {
        let count = u32::from(count) as usize;
        let mut suppressed_error: Option<JsError> = None;

        // Dispose exactly `count` resources in reverse order (LIFO)
        for _ in 0..count {
            if let Some((value, method)) = context.vm.pop_disposable_resource() {
                let result = method.call(&value, &[], context);

                if let Err(err) = result {
                    suppressed_error = Some(match suppressed_error {
                        None => err,
                        Some(previous) => create_suppressed_error(err, &previous, context),
                    });
                }
            }
        }

        if let Some(err) = suppressed_error {
            return Err(err);
        }

        Ok(())
    }
}

impl Operation for DisposeResources {
    const NAME: &'static str = "DisposeResources";
    const INSTRUCTION: &'static str = "INST - DisposeResources";
    const COST: u8 = 5;
}

/// Helper function to create a SuppressedError
fn create_suppressed_error(
    _error: JsError,
    suppressed: &JsError,
    _context: &mut Context,
) -> JsError {
    // For now, we'll create a simple error that contains both errors
    // TODO: Implement proper SuppressedError builtin in Phase 2
    let message = format!("An error was suppressed during disposal: {suppressed}");

    // This is a temporary solution until SuppressedError is implemented
    JsNativeError::error().with_message(message).into()
}
