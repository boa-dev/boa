use crate::{
    builtins::{iterable::IteratorRecord, object::for_in_iterator::ForInIterator},
    error::JsNativeError,
    js_string,
    property::PropertyDescriptor,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
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

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();

        let object = context.vm.pop();
        if object.is_null_or_undefined() {
            context.vm.frame_mut().pc = address as usize;
            return Ok(CompletionType::Normal);
        }

        let object = object.to_object(context)?;
        let iterator = ForInIterator::create_for_in_iterator(JsValue::new(object), context);
        let next_method = iterator
            .get_property(js_string!("next"))
            .as_ref()
            .map(PropertyDescriptor::expect_value)
            .cloned()
            .ok_or_else(|| JsNativeError::typ().with_message("Could not find property `next`"))?;

        context.vm.push(iterator);
        context.vm.push(next_method);
        context.vm.push(false);
        Ok(CompletionType::Normal)
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

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
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
        if let Some(next) = iterator_record.step(context)? {
            context.vm.push(iterator.clone());
            context.vm.push(next_method);
            context.vm.push(done);
            let value = next.value(context)?;
            context.vm.push(value);
        } else {
            context.vm.frame_mut().pc = address as usize;
            context.vm.frame_mut().dec_frame_env_stack();
            context.realm.environments.pop();
            context.vm.push(iterator.clone());
            context.vm.push(next_method);
            context.vm.push(done);
        }
        Ok(CompletionType::Normal)
    }
}
