use crate::{
    vm::{opcode::Operation, CompletionType, Registers},
    Context, JsResult, JsString,
};

/// `ConcatToString` implements the Opcode Operation for `Opcode::ConcatToString`
///
/// Operation:
///  - Concat multiple stack objects into a string.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ConcatToString;

impl ConcatToString {
    fn operation(
        string: u32,
        values: &[u32],
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let mut strings = Vec::with_capacity(values.len());
        for value in values {
            let val = registers.get(*value);
            strings.push(val.to_string(context)?);
        }
        let s = JsString::concat_array(
            &strings
                .iter()
                .map(JsString::as_str)
                .map(Into::into)
                .collect::<Vec<_>>(),
        );
        registers.set(string, s.into());
        Ok(CompletionType::Normal)
    }
}

impl Operation for ConcatToString {
    const NAME: &'static str = "ConcatToString";
    const INSTRUCTION: &'static str = "INST - ConcatToString";
    const COST: u8 = 6;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let string = context.vm.read::<u8>().into();
        let value_count = context.vm.read::<u8>() as usize;
        let mut values = Vec::with_capacity(value_count);
        for _ in 0..value_count {
            values.push(context.vm.read::<u8>().into());
        }
        Self::operation(string, &values, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let string = context.vm.read::<u16>().into();
        let value_count = context.vm.read::<u16>() as usize;
        let mut values = Vec::with_capacity(value_count);
        for _ in 0..value_count {
            values.push(context.vm.read::<u16>().into());
        }
        Self::operation(string, &values, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let string = context.vm.read::<u32>();
        let value_count = context.vm.read::<u32>() as usize;
        let mut values = Vec::with_capacity(value_count);
        for _ in 0..value_count {
            values.push(context.vm.read::<u32>());
        }
        Self::operation(string, &values, registers, context)
    }
}
