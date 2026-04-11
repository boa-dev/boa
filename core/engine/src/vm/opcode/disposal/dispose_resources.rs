use crate::{
    Context, JsError, JsValue,
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
    pub(crate) fn operation(count: IndexOperand, context: &mut Context) -> crate::JsResult<()> {
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
fn create_suppressed_error(error: JsError, suppressed: &JsError, context: &mut Context) -> JsError {
    // Create a proper SuppressedError using the builtin constructor
    // Call SuppressedError(error, suppressed)
    let Ok(error_val) = error.into_opaque(context) else {
        // If we can't convert the error, just return it
        return suppressed.clone();
    };
    let Ok(suppressed_val) = suppressed.clone().into_opaque(context) else {
        return JsError::from_opaque(error_val);
    };

    let args = [error_val, suppressed_val];

    let suppressed_error_constructor = context
        .intrinsics()
        .constructors()
        .suppressed_error()
        .constructor();

    // Call the constructor as a function
    match suppressed_error_constructor.call(&JsValue::undefined(), &args, context) {
        Ok(obj) => JsError::from_opaque(obj),
        Err(e) => e, // Fallback if construction fails
    }
}
