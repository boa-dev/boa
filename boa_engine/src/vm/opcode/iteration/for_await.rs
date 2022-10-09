use crate::{
    builtins::iterable::IteratorResult,
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ForAwaitOfLoopIterate;

impl Operation for ForAwaitOfLoopIterate {
    const NAME: &'static str = "ForAwaitOfLoopIterate";
    const INSTRUCTION: &'static str = "INST - ForAwaitOfLoopIterate";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let _done = context
            .vm
            .pop()
            .as_boolean()
            .expect("iterator [[Done]] was not a boolean");
        let next_method = context.vm.pop();
        let next_method_object = if let Some(object) = next_method.as_callable() {
            object
        } else {
            return context.throw_type_error("iterable next method not a function");
        };
        let iterator = context.vm.pop();
        let next_result = next_method_object.call(&iterator, &[], context)?;
        context.vm.push(iterator);
        context.vm.push(next_method);
        context.vm.push(next_result);
        Ok(ShouldExit::False)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ForAwaitOfLoopNext;

impl Operation for ForAwaitOfLoopNext {
    const NAME: &'static str = "ForAwaitOfLoopNext";
    const INSTRUCTION: &'static str = "INST - ForAwaitOfLoopNext";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let address = context.vm.read::<u32>();

        let next_result = context.vm.pop();
        let next_result = if let Some(next_result) = next_result.as_object() {
            IteratorResult::new(next_result.clone())
        } else {
            return context.throw_type_error("next value should be an object");
        };

        if next_result.complete(context)? {
            context.vm.frame_mut().pc = address as usize;
            context.vm.frame_mut().loop_env_stack_dec();
            context.vm.frame_mut().try_env_stack_dec();
            context.realm.environments.pop();
            context.vm.push(true);
        } else {
            context.vm.push(false);
            let value = next_result.value(context)?;
            context.vm.push(value);
        }
        Ok(ShouldExit::False)
    }
}
