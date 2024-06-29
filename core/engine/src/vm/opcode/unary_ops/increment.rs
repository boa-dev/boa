use crate::{
    value::JsValue,
    vm::{opcode::Operation, CompletionType},
    Context, JsBigInt, JsResult,
};

/// `ToNumeric` implements the Opcode Operation for `Opcode::ToNumeric`
#[derive(Debug, Clone, Copy)]
pub(crate) struct ToNumeric;

impl ToNumeric {
    fn operation(
        src: u32,
        dst: u32,
        operand_types: u8,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let rp = context.vm.frame().rp;
        let value = context
            .vm
            .frame()
            .read_value::<0>(operand_types, src, &context.vm)
            .to_numeric(context)?;
        context.vm.stack[(rp + dst) as usize] = value.into();
        Ok(CompletionType::Normal)
    }
}

impl Operation for ToNumeric {
    const NAME: &'static str = "ToNumeric";
    const INSTRUCTION: &'static str = "INST - ToNumeric";
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let dst = context.vm.read::<u8>().into();
        let src = context.vm.read::<u8>().into();
        Self::operation(src, dst, operand_types, context)
    }
    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let dst = context.vm.read::<u16>().into();
        let src = context.vm.read::<u16>().into();
        Self::operation(src, dst, operand_types, context)
    }
    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let dst = context.vm.read::<u32>();
        let src = context.vm.read::<u32>();
        Self::operation(src, dst, operand_types, context)
    }
}

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
        operand_types: u8,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let rp = context.vm.frame().rp;
        let value = context
            .vm
            .frame()
            .read_value::<0>(operand_types, src, &context.vm);
        let value = match value {
            JsValue::Integer(number) if number < i32::MAX => JsValue::from(number + 1),
            JsValue::Rational(value) => JsValue::from(value + 1f64),
            JsValue::BigInt(bigint) => JsBigInt::add(&bigint, &JsBigInt::one()).into(),
            _ => unreachable!("there is always a call to ToNumeric before Inc"),
        };

        context.vm.stack[(rp + dst) as usize] = value;
        Ok(CompletionType::Normal)
    }
}

impl Operation for Inc {
    const NAME: &'static str = "Inc";
    const INSTRUCTION: &'static str = "INST - Inc";
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let dst = context.vm.read::<u8>().into();
        let src = context.vm.read::<u8>().into();
        Self::operation(src, dst, operand_types, context)
    }
    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let dst = context.vm.read::<u16>().into();
        let src = context.vm.read::<u16>().into();
        Self::operation(src, dst, operand_types, context)
    }
    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let dst = context.vm.read::<u32>();
        let src = context.vm.read::<u32>();
        Self::operation(src, dst, operand_types, context)
    }
}
