use crate::{
    value::Numeric,
    vm::{opcode::Operation, ShouldExit},
    Context, JsBigInt, JsResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Dec;

impl Operation for Dec {
    const NAME: &'static str = "Dec";
    const INSTRUCTION: &'static str = "INST - Dec";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DecPost;

impl Operation for DecPost {
    const NAME: &'static str = "DecPost";
    const INSTRUCTION: &'static str = "INST - DecPost";

    fn execute(context: &mut Context) -> JsResult<ShouldExit> {
        let value = context.vm.pop();
        let value = value.to_numeric(context)?;
        context.vm.push(value.clone());
        match value {
            Numeric::Number(number) => context.vm.push(number - 1f64),
            Numeric::BigInt(bigint) => {
                context.vm.push(JsBigInt::sub(&bigint, &JsBigInt::one()));
            }
        }
        Ok(ShouldExit::False)
    }
}
