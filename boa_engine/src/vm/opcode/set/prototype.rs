use crate::{
    vm::{opcode::Operation, CompletionType},
    Context,
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

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        let object = context.vm.pop();

        let prototype = if let Some(prototype) = value.as_object() {
            Some(prototype.clone())
        } else if value.is_null() {
            None
        } else {
            return CompletionType::Normal;
        };

        let object = object.as_object().expect("object is not an object");
        object
            .__set_prototype_of__(prototype, context)
            .expect("cannot fail per spec");

        CompletionType::Normal
    }
}
