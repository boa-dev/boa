use crate::{
    Context, JsExpect, JsResult,
    builtins::iterable::{IteratorRecord, create_iter_result_object},
    vm::opcode::{IndexOperand, Operation, RegisterOperand},
};

/// `IteratorPop` implements the Opcode Operation for `Opcode::IteratorPop`
///
/// Operation:
///  - Pops the last iterator on the iterators stack.
///
/// Registers (out):
///  - iterator: `JsObject`.
///  - next: `JsValue`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorPop;

impl IteratorPop {
    #[inline(always)]
    pub(crate) fn operation(
        (iterator, next): (RegisterOperand, RegisterOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let iterator_record = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .js_expect("iterator stack should have at least an iterator")?;

        context
            .vm
            .set_register(iterator.into(), iterator_record.iterator().clone().into());
        context
            .vm
            .set_register(next.into(), iterator_record.next_method().clone());

        Ok(())
    }
}

impl Operation for IteratorPop {
    const NAME: &'static str = "IteratorPop";
    const INSTRUCTION: &'static str = "INST - IteratorPop";
    const COST: u8 = 3;
}

/// `IteratorPush` implements the Opcode Operation for `Opcode::IteratorPush`
///
/// Operation:
///  - Pushes an iterator on the iterators stack
///
/// Registers (in):
///  - iterator: `JsObject`.
///  - next: `JsValue`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorPush;

impl IteratorPush {
    #[inline(always)]
    pub(crate) fn operation(
        (iterator, next): (RegisterOperand, RegisterOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let iterator = context
            .vm
            .get_register(iterator.into())
            .as_object()
            .js_expect("iterator should be an object")?;
        let next = context.vm.get_register(next.into()).clone();

        context
            .vm
            .frame_mut()
            .iterators
            .push(IteratorRecord::new(iterator, next));

        Ok(())
    }
}

impl Operation for IteratorPush {
    const NAME: &'static str = "IteratorPush";
    const INSTRUCTION: &'static str = "INST - IteratorPush";
    const COST: u8 = 3;
}

/// `IteratorUpdateResult` implements the Opcode Operation for `Opcode::IteratorUpdateResult`
///
/// Operation:
///  - Updates the result of the currently active iterator.
///
/// Registers (inout):
///  - result: `JsValue` (in), `bool` (out) with the `done` value of the iterator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorUpdateResult;

impl IteratorUpdateResult {
    #[inline(always)]
    pub(crate) fn operation(result: RegisterOperand, context: &mut Context) -> JsResult<()> {
        let mut iterator = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .js_expect("iterator stack should have at least an iterator")?;
        let result_v = context.vm.take_register(result.into());
        iterator.update_result(result_v, context)?;
        context
            .vm
            .set_register(result.into(), iterator.done().into());
        context.vm.frame_mut().iterators.push(iterator);

        Ok(())
    }
}

impl Operation for IteratorUpdateResult {
    const NAME: &'static str = "IteratorUpdateResult";
    const INSTRUCTION: &'static str = "INST - IteratorUpdateResult";
    const COST: u8 = 2;
}

/// `IteratorNext` implements the Opcode Operation for `Opcode::IteratorNext`
///
/// Operation:
///  - Calls the `next` method of `iterator`, updating its record with the next value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorNext;

impl IteratorNext {
    #[inline(always)]
    pub(crate) fn operation((): (), context: &mut Context) -> JsResult<()> {
        let mut iterator = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .js_expect("iterator stack should have at least an iterator")?;

        if let Err(err) = iterator.step(context) {
            context.vm.pending_exception = Some(err);
        }

        context.vm.frame_mut().iterators.push(iterator);

        Ok(())
    }
}

impl Operation for IteratorNext {
    const NAME: &'static str = "IteratorNext";
    const INSTRUCTION: &'static str = "INST - IteratorNext";
    const COST: u8 = 6;
}

/// `IteratorResult` implements the Opcode Operation for `Opcode::IteratorResult`
///
/// Operation:
///  - Gets the last iteration result of the current iterator record.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorResult;

impl IteratorResult {
    #[inline(always)]
    pub(crate) fn operation(value: RegisterOperand, context: &mut Context) -> JsResult<()> {
        let last_result = context
            .vm
            .frame()
            .iterators
            .last()
            .js_expect("iterator on the call frame must exist")?
            .last_result()
            .object()
            .clone();
        context.vm.set_register(value.into(), last_result.into());

        Ok(())
    }
}

impl Operation for IteratorResult {
    const NAME: &'static str = "IteratorResult";
    const INSTRUCTION: &'static str = "INST - IteratorResult";
    const COST: u8 = 3;
}

/// `IteratorValue` implements the Opcode Operation for `Opcode::IteratorValue`
///
/// Operation:
///  - Gets the `value` property of the current iterator record.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorValue;

impl IteratorValue {
    #[inline(always)]
    pub(crate) fn operation(value: RegisterOperand, context: &mut Context) -> JsResult<()> {
        let mut iterator = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .js_expect("iterator stack should have at least an iterator")?;

        match iterator.value(context) {
            Ok(v) => context.vm.set_register(value.into(), v),
            Err(err) => context.vm.pending_exception = Some(err),
        }

        context.vm.frame_mut().iterators.push(iterator);

        Ok(())
    }
}

impl Operation for IteratorValue {
    const NAME: &'static str = "IteratorValue";
    const INSTRUCTION: &'static str = "INST - IteratorValue";
    const COST: u8 = 5;
}

/// `IteratorDone` implements the Opcode Operation for `Opcode::IteratorDone`
///
/// Operation:
///  - Returns `true` if the current iterator is done, or `false` otherwise
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorDone;

impl IteratorDone {
    #[inline(always)]
    pub(crate) fn operation(done: RegisterOperand, context: &mut Context) {
        let value = context
            .vm
            .frame()
            .iterators
            .last()
            .expect("iterator on the call frame must exist")
            .done();
        context.vm.set_register(done.into(), value.into());
    }
}

impl Operation for IteratorDone {
    const NAME: &'static str = "IteratorDone";
    const INSTRUCTION: &'static str = "INST - IteratorDone";
    const COST: u8 = 3;
}

/// `IteratorStackEmpty` implements the Opcode Operation for `Opcode::IteratorStackEmpty`
///
/// Operation:
/// - Store `true` in dst if the iterator stack is empty.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorStackEmpty;

impl IteratorStackEmpty {
    #[inline(always)]
    pub(crate) fn operation(empty: RegisterOperand, context: &mut Context) {
        let is_empty = context.vm.frame().iterators.is_empty();
        context.vm.set_register(empty.into(), is_empty.into());
    }
}

impl Operation for IteratorStackEmpty {
    const NAME: &'static str = "IteratorStackEmpty";
    const INSTRUCTION: &'static str = "INST - IteratorStackEmpty";
    const COST: u8 = 1;
}

/// `CreateIteratorResult` implements the Opcode Operation for `Opcode::CreateIteratorResult`
///
/// Operation:
/// -  Creates a new iterator result object
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateIteratorResult;

impl CreateIteratorResult {
    #[inline(always)]
    pub(crate) fn operation((value, done): (RegisterOperand, IndexOperand), context: &mut Context) {
        let done = u32::from(done) != 0;
        let val = context.vm.take_register(value.into());
        let result = create_iter_result_object(val, done, context);
        context.vm.set_register(value.into(), result);
    }
}

impl Operation for CreateIteratorResult {
    const NAME: &'static str = "CreateIteratorResult";
    const INSTRUCTION: &'static str = "INST - CreateIteratorResult";
    const COST: u8 = 3;
}
