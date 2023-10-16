use crate::{
    vm::{opcode::Operation, CompletionType},
    Context, JsNativeError, JsResult,
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

/// `CheckReturn` implements the Opcode Operation for `Opcode::CheckReturn`
///
/// Operation:
///  - Check return from a function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CheckReturn;

impl Operation for CheckReturn {
    const NAME: &'static str = "CheckReturn";
    const INSTRUCTION: &'static str = "INST - CheckReturn";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        if !context.vm.frame().construct() {
            return Ok(CompletionType::Normal);
        }
        let frame = context.vm.frame();
        let this = frame.this(&context.vm);
        let result = context.vm.take_return_value();

        let result = if result.is_object() {
            result
        } else if !this.is_undefined() {
            this
        } else if !result.is_undefined() {
            let realm = context.vm.frame().realm.clone();
            context.vm.pending_exception = Some(
                JsNativeError::typ()
                    .with_message("derived constructor can only return an Object or undefined")
                    .with_realm(realm)
                    .into(),
            );
            return Ok(CompletionType::Throw);
        } else {
            let realm = context.vm.frame().realm.clone();
            let this = context.vm.environments.get_this_binding();

            match this {
                Err(err) => {
                    let err = err.inject_realm(realm);
                    context.vm.pending_exception = Some(err);
                    return Ok(CompletionType::Throw);
                }
                Ok(this) => this,
            }
        };

        context.vm.set_return_value(result);
        Ok(CompletionType::Normal)
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
