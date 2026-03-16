use crate::{Context, JsError, JsNativeError, JsResult, vm::opcode::Operation};

/// `DisposeResources` implements the DisposeResources operation.
///
/// This opcode disposes all resources in the current disposal stack.
///
/// Operation:
///  - Stack: **=>**
pub(crate) struct DisposeResources;

impl DisposeResources {
    pub(crate) fn operation((): (), context: &mut Context) -> JsResult<()> {
        let mut suppressed_error: Option<JsError> = None;

        // Get the scope depth to know how many resources to dispose
        let scope_depth = context.vm.current_disposal_scope_depth();

        // Dispose resources in reverse order (LIFO) until we reach the scope depth
        while context.vm.disposal_stack.len() > scope_depth {
            if let Some((value, method)) = context.vm.pop_disposable_resource() {
                // Call the dispose method
                let result = method.call(&value, &[], context);

                // If an error occurs, aggregate it
                if let Err(err) = result {
                    suppressed_error = Some(match suppressed_error {
                        None => err,
                        Some(previous) => {
                            // Create a SuppressedError
                            create_suppressed_error(err, &previous, context)
                        }
                    });
                }
            }
        }

        // Pop the disposal scope depth marker
        context.vm.pop_disposal_scope();

        // If there were any errors, throw the aggregated error
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

    // Attach the original error as a property
    // This is a temporary solution until SuppressedError is implemented
    JsNativeError::error().with_message(message).into()
}
