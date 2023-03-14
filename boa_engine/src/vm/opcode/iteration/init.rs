use crate::{
    builtins::iterable::IteratorHint,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `InitIterator` implements the Opcode Operation for `Opcode::InitIterator`
///
/// Operation:
///  - Initialize an iterator
#[derive(Debug, Clone, Copy)]
pub(crate) struct InitIterator;

impl Operation for InitIterator {
    const NAME: &'static str = "InitIterator";
    const INSTRUCTION: &'static str = "INST - InitIterator";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let object = context.vm.pop();
        let iterator = object.get_iterator(context, None, None)?;
        context.vm.push(iterator.iterator().clone());
        context.vm.push(iterator.next_method().clone());
        context.vm.push(iterator.done());
        Ok(CompletionType::Normal)
    }
}

/// `InitAsyncIterator` implements the Opcode Operation for `Opcode::InitAsyncIterator`
///
/// Operation:
///  - Initialize an async iterator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct InitAsyncIterator;

impl Operation for InitAsyncIterator {
    const NAME: &'static str = "InitAsyncIterator";
    const INSTRUCTION: &'static str = "INST - InitAsyncIterator";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let object = context.vm.pop();
        let iterator = object.get_iterator(context, Some(IteratorHint::Async), None)?;
        context.vm.push(iterator.iterator().clone());
        context.vm.push(iterator.next_method().clone());
        Ok(CompletionType::Normal)
    }
}
