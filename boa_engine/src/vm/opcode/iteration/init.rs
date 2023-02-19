use crate::{
    builtins::iterable::IteratorHint,
    vm::{ok_or_throw_completion, opcode::Operation, CompletionType},
    Context,
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

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let object = context.vm.pop();
        let iterator = ok_or_throw_completion!(object.get_iterator(context, None, None), context);
        context.vm.push(iterator.iterator().clone());
        context.vm.push(iterator.next_method().clone());
        context.vm.push(iterator.done());
        CompletionType::Normal
    }
}

/// `InitIteratorAsync` implements the Opcode Operation for `Opcode::InitIteratorAsync`
///
/// Operation:
///  - Initialize an async iterator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct InitIteratorAsync;

impl Operation for InitIteratorAsync {
    const NAME: &'static str = "InitIteratorAsync";
    const INSTRUCTION: &'static str = "INST - InitIteratorAsync";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let object = context.vm.pop();
        let iterator = ok_or_throw_completion!(
            object.get_iterator(context, Some(IteratorHint::Async), None),
            context
        );
        context.vm.push(iterator.iterator().clone());
        context.vm.push(iterator.next_method().clone());
        context.vm.push(iterator.done());
        CompletionType::Normal
    }
}
