use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `SetPrototype` implements the Opcode Operation for `Opcode::SetPrototype`
///
/// Operation:
///  - Sets the prototype of an object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetPrototype;

impl Operation for SetPrototype {
    const NAME: &'static str = "SetPrototype";
    const INSTRUCTION: &'static str = "INST - SetPrototype";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let value = raw_context.vm.pop();
        let object = raw_context.vm.pop();

        let prototype = if let Some(prototype) = value.as_object() {
            Some(prototype.clone())
        } else if value.is_null() {
            None
        } else {
            return Ok(CompletionType::Normal);
        };

        let object = object.as_object().expect("object is not an object");
        object
            .__set_prototype_of__(prototype, context)
            .expect("cannot fail per spec");

        Ok(CompletionType::Normal)
    }
}
