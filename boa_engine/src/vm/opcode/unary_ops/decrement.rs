use crate::{
    value::Numeric,
    vm::{opcode::Operation, ShouldExit},
    Context, JsBigInt, JsResult,
};

/// `Dec` implements the Opcode Operation for `Opcode::Dec`
///
/// Operation:
///  - Unary `--` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Dec;

impl Operation for Dec {
    const NAME: &'static str = "Dec";
    const INSTRUCTION: &'static str = "INST - Dec";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        match value.to_numeric(context)? {
            Numeric::Number(number) => context.vm.push(number - 1f64),
            Numeric::BigInt(bigint) => {
                context.vm.push(JsBigInt::sub(&bigint, &JsBigInt::one()));
            }
        }
        Ok(ShouldExit::False)
    }
}

/// `DecPost` implements the Opcode Operation for `Opcode::DecPost`
///
/// Operation:
///  - Unary postfix `--` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DecPost;

impl Operation for DecPost {
    const NAME: &'static str = "DecPost";
    const INSTRUCTION: &'static str = "INST - DecPost";

    fn execute(context: &mut Context<'_>) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        let value = value.to_numeric(context)?;
        match value {
            Numeric::Number(number) => context.vm.push(number - 1f64),
            Numeric::BigInt(ref bigint) => {
                context.vm.push(JsBigInt::sub(bigint, &JsBigInt::one()));
            }
        }
        context.vm.push(value);
        Ok(ShouldExit::False)
    }
}
