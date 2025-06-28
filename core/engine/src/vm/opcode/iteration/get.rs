use crate::{
    Context, JsResult,
    builtins::iterable::IteratorHint,
    vm::opcode::{Operation, VaryingOperand},
};

/// `GetIterator` implements the Opcode Operation for `Opcode::GetIterator`
///
/// Operation:
///  - Initialize an iterator
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetIterator;

impl GetIterator {
    #[inline(always)]
    pub(crate) fn operation(value: VaryingOperand, context: &mut Context) -> JsResult<()> {
        let value = context.vm.get_register(value.into()).clone();
        let iterator = value.get_iterator(IteratorHint::Sync, context)?;
        context.vm.frame_mut().iterators.push(iterator);
        Ok(())
    }
}

impl Operation for GetIterator {
    const NAME: &'static str = "GetIterator";
    const INSTRUCTION: &'static str = "INST - GetIterator";
    const COST: u8 = 6;
}

/// `GetAsyncIterator` implements the Opcode Operation for `Opcode::GetAsyncIterator`
///
/// Operation:
///  - Initialize an async iterator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetAsyncIterator;

impl GetAsyncIterator {
    #[inline(always)]
    pub(crate) fn operation(value: VaryingOperand, context: &mut Context) -> JsResult<()> {
        let value = context.vm.get_register(value.into()).clone();
        let iterator = value.get_iterator(IteratorHint::Async, context)?;
        context.vm.frame_mut().iterators.push(iterator);
        Ok(())
    }
}

impl Operation for GetAsyncIterator {
    const NAME: &'static str = "GetAsyncIterator";
    const INSTRUCTION: &'static str = "INST - GetAsyncIterator";
    const COST: u8 = 6;
}
