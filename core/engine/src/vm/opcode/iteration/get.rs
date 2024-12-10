use crate::{
    builtins::iterable::IteratorHint,
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

/// `GetIterator` implements the Opcode Operation for `Opcode::GetIterator`
///
/// Operation:
///  - Initialize an iterator
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetIterator;

impl GetIterator {
    fn operation(
        value: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value);
        let iterator = value.get_iterator(IteratorHint::Sync, context)?;
        context.vm.frame_mut().iterators.push(iterator);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetIterator {
    const NAME: &'static str = "GetIterator";
    const INSTRUCTION: &'static str = "INST - GetIterator";
    const COST: u8 = 6;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        Self::operation(value, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        Self::operation(value, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        Self::operation(value, registers, context)
    }
}

/// `GetAsyncIterator` implements the Opcode Operation for `Opcode::GetAsyncIterator`
///
/// Operation:
///  - Initialize an async iterator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetAsyncIterator;

impl GetAsyncIterator {
    fn operation(
        value: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value);
        let iterator = value.get_iterator(IteratorHint::Async, context)?;
        context.vm.frame_mut().iterators.push(iterator);
        Ok(CompletionType::Normal)
    }
}

impl Operation for GetAsyncIterator {
    const NAME: &'static str = "GetAsyncIterator";
    const INSTRUCTION: &'static str = "INST - GetAsyncIterator";
    const COST: u8 = 6;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u8>().into();
        Self::operation(value, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u16>().into();
        Self::operation(value, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.read::<u32>();
        Self::operation(value, registers, context)
    }
}
