use super::VaryingOperand;
use crate::{
    error::JsNativeError,
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult,
};

/// `ValueNotNullOrUndefined` implements the Opcode Operation for `Opcode::ValueNotNullOrUndefined`
///
/// Operation:
///  - Require the stack value to be neither null nor undefined.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ValueNotNullOrUndefined;

impl ValueNotNullOrUndefined {
    pub(super) fn operation(
        value: VaryingOperand,
        registers: &mut Registers,
        _: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(value.into());
        if value.is_null() {
            return Err(JsNativeError::typ()
                .with_message("Cannot destructure 'null' value")
                .into());
        }
        if value.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Cannot destructure 'undefined' value")
                .into());
        }
        Ok(CompletionType::Normal)
    }
}

impl Operation for ValueNotNullOrUndefined {
    const NAME: &'static str = "ValueNotNullOrUndefined";
    const INSTRUCTION: &'static str = "INST - ValueNotNullOrUndefined";
    const COST: u8 = 2;
}

/// `IsObject` implements the Opcode Operation for `Opcode::IsObject`
///
/// Operation:
///  - Pushes `true` to the stack if the top stack value is an object, or `false` otherwise.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IsObject;

impl IsObject {
    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn operation(
        value: VaryingOperand,
        registers: &mut Registers,
        _: &mut Context,
    ) -> JsResult<CompletionType> {
        let is_object = registers.get(value.into()).is_object();
        registers.set(value.into(), is_object.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for IsObject {
    const NAME: &'static str = "IsObject";
    const INSTRUCTION: &'static str = "INST - IsObject";
    const COST: u8 = 1;
}
