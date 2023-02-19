use crate::{
    error::JsNativeError,
    vm::{opcode::Operation, throw_completion, CompletionType},
    Context, JsError,
};

/// `ValueNotNullOrUndefined` implements the Opcode Operation for `Opcode::ValueNotNullOrUndefined`
///
/// Operation:
///  - Require the stack value to be neither null nor undefined.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ValueNotNullOrUndefined;

impl Operation for ValueNotNullOrUndefined {
    const NAME: &'static str = "ValueNotNullOrUndefined";
    const INSTRUCTION: &'static str = "INST - ValueNotNullOrUndefined";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        if value.is_null() {
            throw_completion!(
                JsNativeError::typ()
                    .with_message("Cannot destructure 'null' value")
                    .into(),
                JsError,
                context
            );
        }
        if value.is_undefined() {
            throw_completion!(
                JsNativeError::typ()
                    .with_message("Cannot destructure 'undefined' value")
                    .into(),
                JsError,
                context
            );
        }
        context.vm.push(value);
        CompletionType::Normal
    }
}
