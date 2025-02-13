use crate::{
    builtins::{iterable::IteratorRecord, object::for_in_iterator::ForInIterator},
    js_string,
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult, JsValue,
};

/// `CreateForInIterator` implements the Opcode Operation for `Opcode::CreateForInIterator`
///
/// Operation:
///  - Creates a new `ForInIterator` for the provided object.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateForInIterator;

impl CreateForInIterator {
    fn operation(
        value: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let object = registers.get(value);
        let object = object.to_object(context)?;
        let iterator = ForInIterator::create_for_in_iterator(JsValue::new(object), context);
        let next_method = iterator
            .get(js_string!("next"), context)
            .expect("ForInIterator must have a `next` method");

        context
            .vm
            .frame_mut()
            .iterators
            .push(IteratorRecord::new(iterator, next_method));

        Ok(CompletionType::Normal)
    }
}

impl Operation for CreateForInIterator {
    const NAME: &'static str = "CreateForInIterator";
    const INSTRUCTION: &'static str = "INST - CreateForInIterator";
    const COST: u8 = 4;

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
