use crate::{
    object::JsObject,
    vm::{opcode::Operation, ShouldExit},
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let o = JsObject::with_object_proto(context);
        context.vm.push(o);
        Ok(ShouldExit::False)
    }
}
