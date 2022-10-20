use crate::{
    builtins::{iterable::IteratorRecord, ForInIterator},
    error::JsNativeError,
    property::PropertyDescriptor,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult, JsValue,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ForInLoopInitIterator;

impl Operation for ForInLoopInitIterator {
    const NAME: &'static str = "ForInLoopInitIterator";
    const INSTRUCTION: &'static str = "INST - ForInLoopInitIterator";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let address = context.vm.read::<u32>();

        let object = context.vm.pop();
        if object.is_null_or_undefined() {
            context.vm.frame_mut().pc = address as usize;
            return Ok(ShouldExit::False);
        }

        let object = object.to_object(context)?;
        let iterator = ForInIterator::create_for_in_iterator(JsValue::new(object), context);
        let next_method = iterator
            .get_property("next")
            .as_ref()
            .map(PropertyDescriptor::expect_value)
            .cloned()
            .ok_or_else(|| JsNativeError::typ().with_message("Could not find property `next`"))?;

        context.vm.push(iterator);
        context.vm.push(next_method);
        context.vm.push(false);
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ForInLoopNext;

impl Operation for ForInLoopNext {
    const NAME: &'static str = "ForInLoopInitIterator";
    const INSTRUCTION: &'static str = "INST - ForInLoopInitIterator";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
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
            context.vm.frame_mut().loop_env_stack_dec();
            context.vm.frame_mut().try_env_stack_dec();
            context.realm.environments.pop();
            context.vm.push(iterator.clone());
            context.vm.push(next_method);
            context.vm.push(done);
        }
        Ok(ShouldExit::False)
    }
}
