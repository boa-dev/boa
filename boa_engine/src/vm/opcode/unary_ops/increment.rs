use crate::{
    value::{JsValue, Numeric},
    vm::{opcode::Operation, CompletionType},
    Context, JsBigInt, JsResult,
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

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let value = raw_context.vm.pop();
        match value {
            JsValue::Integer(number) if number < i32::MAX => {
                raw_context.vm.push(number + 1);
            }
            _ => match value.to_numeric(context)? {
                Numeric::Number(number) => context.as_raw_context_mut().vm.push(number + 1f64),
                Numeric::BigInt(bigint) => {
                    context
                        .as_raw_context_mut()
                        .vm
                        .push(JsBigInt::add(&bigint, &JsBigInt::one()));
                }
            },
        }
        Ok(CompletionType::Normal)
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

    fn execute(context: &mut dyn Context<'_>) -> JsResult<CompletionType> {
        let raw_context = context.as_raw_context_mut();
        let value = raw_context.vm.pop();
        match value {
            JsValue::Integer(number) if number < i32::MAX => {
                raw_context.vm.push(number + 1);
                raw_context.vm.push(value);
            }
            _ => {
                let value = value.to_numeric(context)?;
                let context = context.as_raw_context_mut();
                match value {
                    Numeric::Number(number) => context.vm.push(number + 1f64),
                    Numeric::BigInt(ref bigint) => {
                        context.vm.push(JsBigInt::add(bigint, &JsBigInt::one()));
                    }
                }
                context.vm.push(value);
            }
        }
        Ok(CompletionType::Normal)
    }
}
