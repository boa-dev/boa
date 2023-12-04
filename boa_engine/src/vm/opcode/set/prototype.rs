use crate::{
    object::internal_methods::InternalMethodContext,
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
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        let object = context.vm.pop();

        let prototype = if let Some(prototype) = value.as_object() {
            Some(prototype.clone())
        } else if value.is_null() {
            None
        } else {
            return Ok(CompletionType::Normal);
        };

        let object = object.as_object().expect("object is not an object");
        object
            .__set_prototype_of__(prototype, &mut InternalMethodContext::new(context))
            .expect("cannot fail per spec");

        Ok(CompletionType::Normal)
    }
}
