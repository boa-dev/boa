use crate::{
    builtins::{iterable::IteratorRecord, object::for_in_iterator::ForInIterator},
    error::JsNativeError,
    js_string,
    property::PropertyDescriptor,
    vm::{ok_or_throw_completion, opcode::Operation, CompletionType},
    Context, JsValue,
};

/// `ForInLoopInitIterator` implements the Opcode Operation for `Opcode::ForInLoopInitIterator`
///
/// Operation:
///  - Initialize the iterator for a for..in loop or jump to after the loop if object is null or undefined.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ForInLoopInitIterator;

impl Operation for ForInLoopInitIterator {
    const NAME: &'static str = "ForInLoopInitIterator";
    const INSTRUCTION: &'static str = "INST - ForInLoopInitIterator";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let address = context.vm.read::<u32>();

        let object = context.vm.pop();
        if object.is_null_or_undefined() {
            context.vm.frame_mut().pc = address as usize;
            return CompletionType::Normal;
        }

        let object = ok_or_throw_completion!(object.to_object(context), context);
        let iterator = ForInIterator::create_for_in_iterator(JsValue::new(object), context);
        let next_method = ok_or_throw_completion!(
            iterator
                .get_property(js_string!("next"))
                .as_ref()
                .map(PropertyDescriptor::expect_value)
                .cloned()
                .ok_or_else(|| JsNativeError::typ().with_message("Could not find property `next`")),
            context
        );

        context.vm.push(iterator);
        context.vm.push(next_method);
        context.vm.push(false);
        CompletionType::Normal
    }
}

/// `ForInLoopNext` implements the Opcode Operation for `Opcode::ForInLoopNext`
///
/// Operation:
///  - Move to the next value in a for..in loop or jump to exit of the loop if done.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ForInLoopNext;

impl Operation for ForInLoopNext {
    const NAME: &'static str = "ForInLoopInitIterator";
    const INSTRUCTION: &'static str = "INST - ForInLoopInitIterator";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let address = context.vm.read::<u32>();

        let done = context
            .vm
            .pop()
            .as_boolean()
            .expect("iterator [[Done]] was not a boolean");
        let next_method = context.vm.pop();
        let iterator = context.vm.pop();
        let iterator = iterator.as_object().expect("iterator was not an object");

        let iterator_record = IteratorRecord::new(iterator.clone(), next_method.clone(), done);
        if let Some(next) = ok_or_throw_completion!(iterator_record.step(context), context) {
            context.vm.push(iterator.clone());
            context.vm.push(next_method);
            context.vm.push(done);
            let value = ok_or_throw_completion!(next.value(context), context);
            context.vm.push(value);
        } else {
            context.vm.frame_mut().pc = address as usize;
            context.vm.frame_mut().dec_frame_env_stack();
            context.realm.environments.pop();
            context.vm.push(iterator.clone());
            context.vm.push(next_method);
            context.vm.push(done);
        }
        CompletionType::Normal
    }
}
