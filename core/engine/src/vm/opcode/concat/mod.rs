use super::VaryingOperand;
use crate::{
    vm::{opcode::Operation, Registers},
    Context, JsResult, JsString,
};

/// `ConcatToString` implements the Opcode Operation for `Opcode::ConcatToString`
///
/// Operation:
///  - Concat multiple stack objects into a string.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ConcatToString;

impl ConcatToString {
    #[inline(always)]
    pub(super) fn operation(
        (string, values): (VaryingOperand, Vec<VaryingOperand>),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<()> {
        let mut strings = Vec::with_capacity(values.len());
        for value in values {
            let val = registers.get(value.into());
            strings.push(val.to_string(context)?);
        }
        let s = JsString::concat_array(&strings.iter().map(JsString::as_str).collect::<Vec<_>>());
        registers.set(string.into(), s.into());
        Ok(())
    }
}

impl Operation for ConcatToString {
    const NAME: &'static str = "ConcatToString";
    const INSTRUCTION: &'static str = "INST - ConcatToString";
    const COST: u8 = 6;
}
