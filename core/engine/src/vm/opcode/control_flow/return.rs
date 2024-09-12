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
    const COST: u8 = 4;

    fn execute(_context: &mut Context) -> JsResult<CompletionType> {
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
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let frame = context.vm.frame();
        if !frame.construct() {
            return Ok(CompletionType::Normal);
        }
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
            let frame = context.vm.frame();
            if frame.has_this_value_cached() {
                this
            } else {
                let realm = frame.realm.clone();

                match context.vm.environments.get_this_binding() {
                    Err(err) => {
                        let err = err.inject_realm(realm);
                        context.vm.pending_exception = Some(err);
                        return Ok(CompletionType::Throw);
                    }
                    Ok(Some(this)) => this,
                    Ok(None) => context.realm().global_this().clone().into(),
                }
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
    const COST: u8 = 2;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
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
    const COST: u8 = 2;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        context.vm.set_return_value(value);
        Ok(CompletionType::Normal)
    }
}

/// TODO: doc
#[derive(Debug, Clone, Copy)]
pub(crate) struct PopIntoRegister;

impl PopIntoRegister {
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::needless_pass_by_value)]
    fn operation(dst: u32, context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.pop();

        let rp = context.vm.frame().rp;
        context.vm.stack[(rp + dst) as usize] = value;
        Ok(CompletionType::Normal)
    }
}

impl Operation for PopIntoRegister {
    const NAME: &'static str = "PopIntoRegister";
    const INSTRUCTION: &'static str = "INST - PopIntoRegister";
    const COST: u8 = 2;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u8>());
        Self::operation(dst, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u16>());
        Self::operation(dst, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u32>();
        Self::operation(dst, context)
    }
}

/// TODO: doc
#[derive(Debug, Clone, Copy)]
pub(crate) struct PushFromRegister;

impl PushFromRegister {
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::needless_pass_by_value)]
    fn operation(dst: u32, context: &mut Context) -> JsResult<CompletionType> {
        let rp = context.vm.frame().rp;
        let value = context.vm.stack[(rp + dst) as usize].clone();
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for PushFromRegister {
    const NAME: &'static str = "PushFromRegister";
    const INSTRUCTION: &'static str = "INST - PushFromRegister";
    const COST: u8 = 2;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u8>());
        Self::operation(dst, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u16>());
        Self::operation(dst, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u32>();
        Self::operation(dst, context)
    }
}
