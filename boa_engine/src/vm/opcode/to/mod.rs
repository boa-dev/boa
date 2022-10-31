use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

/// `ToBoolean` implements the Opcode Operation for `Opcode::ToBoolean`
///
/// Operation:
///  - Pops value converts it to boolean and pushes it back.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ToBoolean;

impl Operation for ToBoolean {
    const NAME: &'static str = "ToBoolean";
    const INSTRUCTION: &'static str = "INST - ToBoolean";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        context.vm.push(value.to_boolean());
        Ok(ShouldExit::False)
    }
}

/// `ToPropertyKey` implements the Opcode Operation for `Opcode::ToPropertyKey`
///
/// Operation:
///  - Call `ToPropertyKey` on the value on the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ToPropertyKey;

impl Operation for ToPropertyKey {
    const NAME: &'static str = "ToPropertyKey";
    const INSTRUCTION: &'static str = "INST - ToPropertyKey";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        let key = value.to_property_key(context)?;
        context.vm.push(key);
        Ok(ShouldExit::False)
    }
}
