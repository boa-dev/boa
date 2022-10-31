use crate::{
    builtins::iterable::IteratorHint,
    vm::{opcode::Operation, ShouldExit},
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let object = context.vm.pop();
        let iterator = object.get_iterator(context, None, None)?;
        context.vm.push(iterator.iterator().clone());
        context.vm.push(iterator.next_method().clone());
        context.vm.push(iterator.done());
        Ok(ShouldExit::False)
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

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let object = context.vm.pop();
        let iterator = object.get_iterator(context, Some(IteratorHint::Async), None)?;
        context.vm.push(iterator.iterator().clone());
        context.vm.push(iterator.next_method().clone());
        context.vm.push(iterator.done());
        Ok(ShouldExit::False)
    }
}
