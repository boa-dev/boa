use crate::value::JsVariant;
use crate::{
    value::Numeric,
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
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        match value.variant() {
            JsVariant::Integer32(number) if number < i32::MAX => {
                context.vm.push(number + 1);
            }
            _ => match value.to_numeric(context)? {
                Numeric::Number(number) => context.vm.push(number + 1f64),
                Numeric::BigInt(bigint) => {
                    context.vm.push(JsBigInt::add(&bigint, &JsBigInt::one()));
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
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        match value.variant() {
            JsVariant::Integer32(number) if number < i32::MAX => {
                context.vm.push(number + 1);
                context.vm.push(value);
            }
            _ => {
                let value = value.to_numeric(context)?;
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
