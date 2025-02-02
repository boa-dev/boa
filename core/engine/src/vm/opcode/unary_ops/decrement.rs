use crate::value::JsVariant;
use crate::{
    value::Numeric,
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
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        match value.variant() {
            JsVariant::Integer32(number) if number > i32::MIN => {
                context.vm.push(number - 1);
            }
            _ => match value.to_numeric(context)? {
                Numeric::Number(number) => context.vm.push(number - 1f64),
                Numeric::BigInt(bigint) => {
                    context.vm.push(JsBigInt::sub(&bigint, &JsBigInt::one()));
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
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let value = context.vm.pop();
        match value.variant() {
            JsVariant::Integer32(number) if number > i32::MIN => {
                context.vm.push(number - 1);
                context.vm.push(value);
            }
            _ => {
                let value = value.to_numeric(context)?;
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
