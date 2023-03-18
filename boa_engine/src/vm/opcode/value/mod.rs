use crate::{
    error::JsNativeError,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
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

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        if value.is_null() {
            return Err(JsNativeError::typ()
                .with_message("Cannot destructure 'null' value")
                .into());
        }
        if value.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Cannot destructure 'undefined' value")
                .into());
        }
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

/// `IsObject` implements the Opcode Operation for `Opcode::IsObject`
///
/// Operation:
///  - Pushes `true` to the stack if the top stack value is an object, or `false` otherwise.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IsObject;

impl Operation for IsObject {
    const NAME: &'static str = "IsObject";
    const INSTRUCTION: &'static str = "INST - IsObject";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        context.vm.push(value.is_object());
        Ok(CompletionType::Normal)
    }
}
