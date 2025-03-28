use crate::{
    value::{JsValue, JsVariant, Numeric},
    vm::{
        opcode::{Operation, VaryingOperand},
        CompletionType, Registers,
    },
    Context, JsBigInt, JsResult,
};

/// `Inc` implements the Opcode Operation for `Opcode::Inc`
///
/// Operation:
///  - Unary `++` operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Inc;

impl Inc {
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn operation(
        (src, dst): (VaryingOperand, VaryingOperand),
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(src.into());

        let (numeric, value) = match value.variant() {
            JsVariant::Integer32(number) if number < i32::MAX => {
                (JsValue::from(number), JsValue::from(number + 1))
            }
            _ => match value.to_numeric(context)? {
                Numeric::Number(number) => (JsValue::from(number), JsValue::from(number + 1f64)),
                Numeric::BigInt(bigint) => (
                    JsValue::from(bigint.clone()),
                    JsValue::from(JsBigInt::add(&bigint, &JsBigInt::one())),
                ),
            },
        };
        registers.set(src.into(), numeric);
        registers.set(dst.into(), value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for Inc {
    const NAME: &'static str = "Inc";
    const INSTRUCTION: &'static str = "INST - Inc";
    const COST: u8 = 3;
}
