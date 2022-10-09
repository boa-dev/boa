use crate::{
    vm::{opcode::Operation, ShouldExit},
    Context, JsResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ValueNotNullOrUndefined;

impl Operation for ValueNotNullOrUndefined {
    const NAME: &'static str = "ValueNotNullOrUndefined";
    const INSTRUCTION: &'static str = "INST - ValueNotNullOrUndefined";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        if value.is_null() {
            return context.throw_type_error("Cannot destructure 'null' value");
        }
        if value.is_undefined() {
            return context.throw_type_error("Cannot destructure 'undefined' value");
        }
        context.vm.push(value);
        Ok(ShouldExit::False)
    }
}
