use crate::{
    builtins::{iterable::IteratorRecord, Array},
    vm::{ok_or_throw_completion, opcode::Operation, CompletionType},
    Context, JsValue,
};

/// `IteratorNext` implements the Opcode Operation for `Opcode::IteratorNext`
///
/// Operation:
///  - Advance the iterator by one and put the value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorNext;

impl Operation for IteratorNext {
    const NAME: &'static str = "IteratorNext";
    const INSTRUCTION: &'static str = "INST - IteratorNext";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let done = context
            .vm
            .pop()
            .as_boolean()
            .expect("iterator [[Done]] was not a boolean");
        let next_method = context.vm.pop();
        let iterator = context.vm.pop();
        let iterator = iterator.as_object().expect("iterator was not an object");

        let iterator_record = IteratorRecord::new(iterator.clone(), next_method.clone(), done);
        let next = ok_or_throw_completion!(iterator_record.step(context), context);

        context.vm.push(iterator.clone());
        context.vm.push(next_method);
        if let Some(next) = next {
            let value = ok_or_throw_completion!(next.value(context), context);
            context.vm.push(false);
            context.vm.push(value);
        } else {
            context.vm.push(true);
            context.vm.push(JsValue::undefined());
        }
        CompletionType::Normal
    }
}

/// `IteratorClose` implements the Opcode Operation for `Opcode::IteratorClose`
///
/// Operation:
///  - Close an iterator
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorClose;

impl Operation for IteratorClose {
    const NAME: &'static str = "IteratorClose";
    const INSTRUCTION: &'static str = "INST - IteratorClose";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let done = context
            .vm
            .pop()
            .as_boolean()
            .expect("iterator [[Done]] was not a boolean");
        let next_method = context.vm.pop();
        let iterator = context.vm.pop();
        let iterator = iterator.as_object().expect("iterator was not an object");
        if !done {
            let iterator_record = IteratorRecord::new(iterator.clone(), next_method, done);
            ok_or_throw_completion!(iterator_record.close(Ok(JsValue::Null), context), context);
        }
        CompletionType::Normal
    }
}

/// `IteratorToArray` implements the Opcode Operation for `Opcode::IteratorToArray`
///
/// Operation:
///  - Consume the iterator and construct and array with all the values.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorToArray;

impl Operation for IteratorToArray {
    const NAME: &'static str = "IteratorToArray";
    const INSTRUCTION: &'static str = "INST - IteratorToArray";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let done = context
            .vm
            .pop()
            .as_boolean()
            .expect("iterator [[Done]] was not a boolean");
        let next_method = context.vm.pop();
        let iterator = context.vm.pop();
        let iterator = iterator.as_object().expect("iterator was not an object");

        let iterator_record = IteratorRecord::new(iterator.clone(), next_method.clone(), done);
        let mut values = Vec::new();

        while let Some(result) = ok_or_throw_completion!(iterator_record.step(context), context) {
            values.push(ok_or_throw_completion!(result.value(context), context));
        }

        let array = Array::create_array_from_list(values, context);

        context.vm.push(iterator.clone());
        context.vm.push(next_method);
        context.vm.push(true);
        context.vm.push(array);
        CompletionType::Normal
    }
}
