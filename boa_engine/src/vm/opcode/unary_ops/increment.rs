use crate::{
    value::{JsValue, Numeric},
    vm::{ok_or_throw_completion, opcode::Operation, CompletionType},
    Context, JsBigInt,
};

/// `Inc` implements the Opcode Operation for `Opcode::Inc`
///
/// Operation:
///  - Unary `++` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Inc;

impl Operation for Inc {
    const NAME: &'static str = "Inc";
    const INSTRUCTION: &'static str = "INST - Inc";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        match value {
            JsValue::Integer(number) if number < i32::MAX => {
                context.vm.push(number + 1);
            }
            _ => match ok_or_throw_completion!(value.to_numeric(context), context) {
                Numeric::Number(number) => context.vm.push(number + 1f64),
                Numeric::BigInt(bigint) => {
                    context.vm.push(JsBigInt::add(&bigint, &JsBigInt::one()));
                }
            },
        }
        CompletionType::Normal
    }
}

/// `Inc` implements the Opcode Operation for `Opcode::Inc`
///
/// Operation:
///  - Unary postfix `++` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IncPost;

impl Operation for IncPost {
    const NAME: &'static str = "IncPost";
    const INSTRUCTION: &'static str = "INST - IncPost";

    fn execute(context: &mut Context<'_>) -> CompletionType {
        let value = context.vm.pop();
        match value {
            JsValue::Integer(number) if number < i32::MAX => {
                context.vm.push(number + 1);
                context.vm.push(value);
            }
            _ => {
                let value = ok_or_throw_completion!(value.to_numeric(context), context);
                match value {
                    Numeric::Number(number) => context.vm.push(number + 1f64),
                    Numeric::BigInt(ref bigint) => {
                        context.vm.push(JsBigInt::add(bigint, &JsBigInt::one()));
                    }
                }
                context.vm.push(value);
            }
        }
        CompletionType::Normal
    }
}
