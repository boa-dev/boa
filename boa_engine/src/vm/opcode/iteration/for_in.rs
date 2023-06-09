use crate::{
    builtins::{iterable::IteratorRecord, object::for_in_iterator::ForInIterator},
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
};

/// `CreateForInIterator` implements the Opcode Operation for `Opcode::CreateForInIterator`
///
/// Operation:
///  - Creates a new `ForInIterator` for the provided object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateForInIterator;

impl Operation for CreateForInIterator {
    const NAME: &'static str = "CreateForInIterator";
    const INSTRUCTION: &'static str = "INST - CreateForInIterator";

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let object = context.as_raw_context_mut().vm.pop();

        let object = object.to_object(context)?;
        let iterator = ForInIterator::create_for_in_iterator(JsValue::new(object), context);
        let next_method = iterator
            .get("next", context)
            .expect("ForInIterator must have a `next` method");

        context
            .as_raw_context_mut()
            .vm
            .frame_mut()
            .iterators
            .push(IteratorRecord::new(iterator, next_method));

        Ok(CompletionType::Normal)
    }
}
