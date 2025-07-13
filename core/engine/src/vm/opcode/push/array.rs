use thin_vec::ThinVec;

use crate::{
    Context, JsExpect, JsResult,
    builtins::Array,
    string::StaticJsStrings,
    vm::opcode::{Operation, RegisterOperand},
};

/// `StoreNewArray` implements the Opcode Operation for `Opcode::StoreNewArray`
///
/// Operation:
///  - Store an empty array in dst.
#[derive(Debug, Clone, Copy)]
pub(crate) struct StoreNewArray;

impl StoreNewArray {
    #[inline(always)]
    pub(crate) fn operation(array: RegisterOperand, context: &mut Context) {
        let value = context
            .intrinsics()
            .templates()
            .array()
            .create(Array, ThinVec::default());
        context.vm.set_register(array.into(), value.into());
    }
}

impl Operation for StoreNewArray {
    const NAME: &'static str = "StoreNewArray";
    const INSTRUCTION: &'static str = "INST - StoreNewArray";
    const COST: u8 = 3;
}

/// `PushValueToArray` implements the Opcode Operation for `Opcode::PushValueToArray`
///
/// Operation:
///  - Push a value to an array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushValueToArray;

impl PushValueToArray {
    #[inline(always)]
    pub(crate) fn operation(
        (value, array): (RegisterOperand, RegisterOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let value = context.vm.get_register(value.into()).clone();
        let o = context
            .vm
            .get_register(array.into())
            .as_object()
            .js_expect("should be an object")?;

        let mut o_borrow = o.borrow_mut();
        let key = o_borrow.properties().indexed_properties.array_length();
        o_borrow
            .properties_mut()
            .indexed_properties
            .insert_value(key, &value);
        Ok(())
    }
}

impl Operation for PushValueToArray {
    const NAME: &'static str = "PushValueToArray";
    const INSTRUCTION: &'static str = "INST - PushValueToArray";
    const COST: u8 = 3;
}

/// `PushElisionToArray` implements the Opcode Operation for `Opcode::PushElisionToArray`
///
/// Operation:
///  - Push an empty element/hole to an array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushElisionToArray;

impl PushElisionToArray {
    #[inline(always)]
    pub(crate) fn operation(array: RegisterOperand, context: &mut Context) -> JsResult<()> {
        let array = context.vm.get_register(array.into()).clone();
        let o = array.as_object().js_expect("should always be an object")?;
        let len = o
            .length_of_array_like(context)
            .js_expect("arrays should always have a 'length' property")?;
        o.set(StaticJsStrings::LENGTH, len + 1, true, context)?;
        o.borrow_mut()
            .properties_mut()
            .indexed_properties
            .transform_to_sparse();
        Ok(())
    }
}

impl Operation for PushElisionToArray {
    const NAME: &'static str = "PushElisionToArray";
    const INSTRUCTION: &'static str = "INST - PushElisionToArray";
    const COST: u8 = 3;
}

/// `PushIteratorToArray` implements the Opcode Operation for `Opcode::PushIteratorToArray`
///
/// Operation:
///  - Push all iterator values to an array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushIteratorToArray;

impl PushIteratorToArray {
    #[inline(always)]
    pub(crate) fn operation(array: RegisterOperand, context: &mut Context) -> JsResult<()> {
        let array = context.vm.get_register(array.into()).clone();
        let mut iterator = context
            .vm
            .frame_mut()
            .iterators
            .pop()
            .js_expect("iterator stack should have at least an iterator")?;
        while let Some(next) = iterator.step_value(context)? {
            Array::push(&array, &[next], context)?;
        }
        Ok(())
    }
}

impl Operation for PushIteratorToArray {
    const NAME: &'static str = "PushIteratorToArray";
    const INSTRUCTION: &'static str = "INST - PushIteratorToArray";
    const COST: u8 = 8;
}
