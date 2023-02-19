use crate::{
    value::{JsValue, Numeric},
    vm::{ok_or_throw_completion, opcode::Operation, CompletionType},
    Context, JsBigInt,
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

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        match value {
            JsValue::Integer(number) if number > i32::MIN => {
                context.vm.push(number - 1);
            }
            _ => match ok_or_throw_completion!(value.to_numeric(context), context) {
                Numeric::Number(number) => context.vm.push(number - 1f64),
                Numeric::BigInt(bigint) => {
                    context.vm.push(JsBigInt::sub(&bigint, &JsBigInt::one()));
                }
            },
        }
        CompletionType::Normal
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

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        match value {
            JsValue::Integer(number) if number > i32::MIN => {
                context.vm.push(number - 1);
                context.vm.push(value);
            }
            _ => {
                let value = ok_or_throw_completion!(value.to_numeric(context), context);
                match value {
                    Numeric::Number(number) => context.vm.push(number - 1f64),
                    Numeric::BigInt(ref bigint) => {
                        context.vm.push(JsBigInt::sub(bigint, &JsBigInt::one()));
                    }
                }
                context.vm.push(value);
            }
        }
        CompletionType::Normal
    }
}
