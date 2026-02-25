use super::VaryingOperand;
use crate::{Context, JsError, JsNativeError, JsResult, JsString, vm::opcode::Operation};
use thin_vec::ThinVec;

/// `ConcatToString` implements the Opcode Operation for `Opcode::ConcatToString`
///
/// Operation:
///  - Concat multiple stack objects into a string.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ConcatToString;

impl ConcatToString {
    #[inline(always)]
    pub(super) fn operation(
        (string, values): (VaryingOperand, ThinVec<VaryingOperand>),
        context: &mut Context,
    ) -> JsResult<()> {
        let mut strings = Vec::with_capacity(values.len());
        for value in values {
            let val = context.vm.get_register(value.into()).clone();
            strings.push(val.to_string(context)?);
        }
        // Use fallible concat
        let s =
            JsString::try_concat_array(&strings.iter().map(JsString::as_str).collect::<Vec<_>>())
                .map_err(|e| {
                JsError::from(
                    // ← ADD JsError::from
                    JsNativeError::range().with_message(format!("Invalid string length: {e}")),
                )
            })?;
        context.vm.set_register(string.into(), s.into());
        Ok(())
    }
}

impl Operation for ConcatToString {
    const NAME: &'static str = "ConcatToString";
    const INSTRUCTION: &'static str = "INST - ConcatToString";
    const COST: u8 = 6;
}
