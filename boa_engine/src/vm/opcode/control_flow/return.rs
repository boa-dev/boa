use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsResult,
};

/// `Return` implements the Opcode Operation for `Opcode::Return`
///
/// Operation:
///  - Return from a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Return;

impl Operation for Return {
    const NAME: &'static str = "Return";
    const INSTRUCTION: &'static str = "INST - Return";

    fn execute(_context: &mut Context<'_>) -> JsResult<CompletionType> {
        Ok(CompletionType::Return)
    }
}

/// `GetReturnValue` implements the Opcode Operation for `Opcode::GetReturnValue`
///
/// Operation:
///  - Gets the return value of a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GetReturnValue;

impl Operation for GetReturnValue {
    const NAME: &'static str = "GetReturnValue";
    const INSTRUCTION: &'static str = "INST - GetReturnValue";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value = context.vm.get_return_value();
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

/// `SetReturnValue` implements the Opcode Operation for `Opcode::SetReturnValue`
///
/// Operation:
///  - Sets the return value of a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetReturnValue;

impl Operation for SetReturnValue {
    const NAME: &'static str = "SetReturnValue";
    const INSTRUCTION: &'static str = "INST - SetReturnValue";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        context.vm.set_return_value(value);
        Ok(CompletionType::Normal)
    }
}
