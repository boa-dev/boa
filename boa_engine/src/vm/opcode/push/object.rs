use crate::{
    object::ObjectData,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `PushEmptyObject` implements the Opcode Operation for `Opcode::PushEmptyObject`
///
/// Operation:
///  - Push empty object `{}` value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushEmptyObject;

impl Operation for PushEmptyObject {
    const NAME: &'static str = "PushEmptyObject";
    const INSTRUCTION: &'static str = "INST - PushEmptyObject";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let o = context
            .intrinsics()
            .templates()
            .ordinary_object()
            .create(ObjectData::ordinary(), Vec::default());
        context.vm.push(o);
        Ok(CompletionType::Normal)
    }
}
