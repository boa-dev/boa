use crate::{
    builtins::{iterable::IteratorRecord, Array},
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `PushNewArray` implements the Opcode Operation for `Opcode::PushNewArray`
///
/// Operation:
///  - Push an empty array value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushNewArray;

impl Operation for PushNewArray {
    const NAME: &'static str = "PushNewArray";
    const INSTRUCTION: &'static str = "INST - PushNewArray";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let array = Array::array_create(0, None, context)
            .expect("Array creation with 0 length should never fail");
        context.vm.push(array);
        Ok(ShouldExit::False)
    }
}

/// `PushValueToArray` implements the Opcode Operation for `Opcode::PushValueToArray`
///
/// Operation:
///  - Push a value to an array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushValueToArray;

impl Operation for PushValueToArray {
    const NAME: &'static str = "PushValueToArray";
    const INSTRUCTION: &'static str = "INST - PushValueToArray";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        let array = context.vm.pop();
        let o = array.as_object().expect("should be an object");
        let len = o
            .length_of_array_like(context)
            .expect("should have 'length' property");
        o.create_data_property_or_throw(len, value, context)
            .expect("should be able to create new data property");
        context.vm.push(array);
        Ok(ShouldExit::False)
    }
}

/// `PushEllisionToArray` implements the Opcode Operation for `Opcode::PushEllisionToArray`
///
/// Operation:
///  - Push an empty element/hole to an array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushElisionToArray;

impl Operation for PushElisionToArray {
    const NAME: &'static str = "PushElisionToArray";
    const INSTRUCTION: &'static str = "INST - PushElisionToArray";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let array = context.vm.pop();
        let o = array.as_object().expect("should always be an object");

        let len = o
            .length_of_array_like(context)
            .expect("arrays should always have a 'length' property");

        o.set("length", len + 1, true, context)?;
        context.vm.push(array);
        Ok(ShouldExit::False)
    }
}

/// `PushIteratorToArray` implements the Opcode Operation for `Opcode::PushIteratorToArray`
///
/// Operation:
///  - Push all iterator values to an array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushIteratorToArray;

impl Operation for PushIteratorToArray {
    const NAME: &'static str = "PushIteratorToArray";
    const INSTRUCTION: &'static str = "INST - PushIteratorToArray";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let done = context
            .vm
            .pop()
            .as_boolean()
            .expect("iterator [[Done]] was not a boolean");
        let next_method = context.vm.pop();
        let iterator = context.vm.pop();
        let iterator = iterator.as_object().expect("iterator was not an object");
        let array = context.vm.pop();

        let iterator = IteratorRecord::new(iterator.clone(), next_method, done);
        while let Some(next) = iterator.step(context)? {
            Array::push(&array, &[next.value(context)?], context)?;
        }

        context.vm.push(array);
        Ok(ShouldExit::False)
    }
}
