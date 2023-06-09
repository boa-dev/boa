use crate::{
    value::{JsValue, Numeric},
    vm::{opcode::Operation, CompletionType},
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

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let value = raw_context.vm.pop();
        match value {
            JsValue::Integer(number) if number > i32::MIN => {
                raw_context.vm.push(number - 1);
            }
            _ => match value.to_numeric(context)? {
                Numeric::Number(number) => context.as_raw_context_mut().vm.push(number - 1f64),
                Numeric::BigInt(bigint) => {
                    context
                        .as_raw_context_mut()
                        .vm
                        .push(JsBigInt::sub(&bigint, &JsBigInt::one()));
                }
            },
        }
        Ok(CompletionType::Normal)
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

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let value = raw_context.vm.pop();
        match value {
            JsValue::Integer(number) if number > i32::MIN => {
                raw_context.vm.push(number - 1);
                raw_context.vm.push(value);
            }
            _ => {
                let value = value.to_numeric(context)?;
                let context = context.as_raw_context_mut();
                match value {
                    Numeric::Number(number) => context.vm.push(number - 1f64),
                    Numeric::BigInt(ref bigint) => {
                        context.vm.push(JsBigInt::sub(bigint, &JsBigInt::one()));
                    }
                }
                context.vm.push(value);
            }
        }
        Ok(CompletionType::Normal)
    }
}
