use crate::{
    builtins::iterable::IteratorHint,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `GetIterator` implements the Opcode Operation for `Opcode::GetIterator`
///
/// Operation:
///  - Initialize an iterator
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetIterator;

impl Operation for GetIterator {
    const NAME: &'static str = "GetIterator";
    const INSTRUCTION: &'static str = "INST - GetIterator";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let object = context.vm.pop();
        let iterator = object.get_iterator(context, None, None)?;
        context.vm.frame_mut().iterators.push(iterator);
        Ok(CompletionType::Normal)
    }
}

/// `GetAsyncIterator` implements the Opcode Operation for `Opcode::GetAsyncIterator`
///
/// Operation:
///  - Initialize an async iterator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetAsyncIterator;

impl Operation for GetAsyncIterator {
    const NAME: &'static str = "GetAsyncIterator";
    const INSTRUCTION: &'static str = "INST - GetAsyncIterator";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let object = context.vm.pop();
        let iterator = object.get_iterator(context, Some(IteratorHint::Async), None)?;
        context.vm.frame_mut().iterators.push(iterator);
        Ok(CompletionType::Normal)
    }
}
