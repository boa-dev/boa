use super::VaryingOperand;
use crate::{error::JsNativeError, vm::opcode::Operation, Context, JsResult};

/// `ValueNotNullOrUndefined` implements the Opcode Operation for `Opcode::ValueNotNullOrUndefined`
///
/// Operation:
///  - Require the stack value to be neither null nor undefined.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ValueNotNullOrUndefined;

impl ValueNotNullOrUndefined {
    #[inline(always)]
    pub(super) fn operation(value: VaryingOperand, context: &mut Context) -> JsResult<()> {
        let value = context.vm.get_register(value.into());
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
        Ok(())
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
    #[inline(always)]
    pub(super) fn operation(value: VaryingOperand, context: &mut Context) {
        let is_object = context.vm.get_register(value.into()).is_object();
        context.vm.set_register(value.into(), is_object.into());
    }
}

impl Operation for IsObject {
    const NAME: &'static str = "IsObject";
    const INSTRUCTION: &'static str = "INST - IsObject";
    const COST: u8 = 1;
}
