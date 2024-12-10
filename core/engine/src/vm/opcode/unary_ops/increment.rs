use crate::{
    value::JsVariant,
    value::{JsValue, Numeric},
    vm::{opcode::Operation, CompletionType, Registers},
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
    fn operation(
        src: u32,
        dst: u32,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let value = registers.get(src);

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
        registers.set(src, numeric);
        registers.set(dst, value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for Inc {
    const NAME: &'static str = "Inc";
    const INSTRUCTION: &'static str = "INST - Inc";
    const COST: u8 = 3;

    fn execute(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u8>().into();
        let src = context.vm.read::<u8>().into();
        Self::operation(src, dst, registers, context)
    }

    fn execute_u16(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u16>().into();
        let src = context.vm.read::<u16>().into();
        Self::operation(src, dst, registers, context)
    }

    fn execute_u32(registers: &mut Registers, context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u32>();
        let src = context.vm.read::<u32>();
        Self::operation(src, dst, registers, context)
    }
}
