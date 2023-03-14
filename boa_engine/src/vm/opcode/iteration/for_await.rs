use crate::{
    builtins::iterable::IteratorResult,
    error::JsNativeError,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `ForAwaitOfLoopIterate` implements the Opcode Operation for `Opcode::ForAwaitOfLoopIterator`
///
/// Operation:
///  - Move to the next value in a for await..of loop.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ForAwaitOfLoopIterate;

impl Operation for ForAwaitOfLoopIterate {
    const NAME: &'static str = "ForAwaitOfLoopIterate";
    const INSTRUCTION: &'static str = "INST - ForAwaitOfLoopIterate";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let next_method = context.vm.pop();
        let iterator = context.vm.pop();
        let next_result = next_method.call(&iterator, &[], context)?;
        context.vm.push(iterator);
        context.vm.push(next_method);
        context.vm.push(next_result);
        Ok(CompletionType::Normal)
    }
}

/// `ForAwaitOfLoopNext` implements the Opcode Operation for `Opcode::ForAwaitOfLoopNext`
///
/// Operation:
///  - Get the value from a for await..of loop next result.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ForAwaitOfLoopNext;

impl Operation for ForAwaitOfLoopNext {
    const NAME: &'static str = "ForAwaitOfLoopNext";
    const INSTRUCTION: &'static str = "INST - ForAwaitOfLoopNext";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let address = context.vm.read::<u32>();

        let next_result = context.vm.pop();
        let next_result = next_result
            .as_object()
            .cloned()
            .map(IteratorResult::new)
            .ok_or_else(|| JsNativeError::typ().with_message("next value should be an object"))?;

        if next_result.complete(context)? {
            context.vm.frame_mut().pc = address as usize;
            context.vm.frame_mut().dec_frame_env_stack();
            context.realm.environments.pop();
        } else {
            let value = next_result.value(context)?;
            context.vm.push(value);
        }
        Ok(CompletionType::Normal)
    }
}
