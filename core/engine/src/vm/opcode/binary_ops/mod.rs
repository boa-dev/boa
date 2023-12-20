use crate::{
    error::JsNativeError,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
};

pub(crate) mod logical;
pub(crate) mod macro_defined;

pub(crate) use logical::*;
pub(crate) use macro_defined::*;

/// `NotEq` implements the Opcode Operation for `Opcode::NotEq`
///
/// Operation:
///  - Binary `!=` operation
#[derive(Debug, Clone, Copy)]
pub(crate) struct NotEq;

impl NotEq {
    #[allow(clippy::needless_pass_by_value)]
    fn operation(
        output: u32,
        lhs: u32,
        rhs: u32,
        operand_types: u8,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let rp = context.vm.frame().rp;

        let lhs = context
            .vm
            .frame()
            .read_value::<0>(operand_types, lhs, &context.vm);
        let rhs = context
            .vm
            .frame()
            .read_value::<1>(operand_types, rhs, &context.vm);

        let value = !lhs.equals(&rhs, context)?;

        context.vm.stack[(rp + output) as usize] = JsValue::from(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for NotEq {
    const NAME: &'static str = "NotEq";
    const INSTRUCTION: &'static str = "INST - NotEq";
    const COST: u8 = 2;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let output = context.vm.read::<u8>().into();
        let lhs = context.vm.read::<u8>().into();
        let rhs = context.vm.read::<u8>().into();
        Self::operation(output, lhs, rhs, operand_types, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let output = context.vm.read::<u16>().into();
        let lhs = context.vm.read::<u16>().into();
        let rhs = context.vm.read::<u16>().into();
        Self::operation(output, lhs, rhs, operand_types, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let output = context.vm.read::<u32>();
        let lhs = context.vm.read::<u32>();
        let rhs = context.vm.read::<u32>();
        Self::operation(output, lhs, rhs, operand_types, context)
    }
}

/// `StrictEq` implements the Opcode Operation for `Opcode::StrictEq`
///
/// Operation:
///  - Binary `===` operation
#[derive(Debug, Clone, Copy)]
pub(crate) struct StrictEq;

impl StrictEq {
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::needless_pass_by_value)]
    fn operation(
        output: u32,
        lhs: u32,
        rhs: u32,
        operand_types: u8,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let rp = context.vm.frame().rp;

        let lhs = context
            .vm
            .frame()
            .read_value::<0>(operand_types, lhs, &context.vm);
        let rhs = context
            .vm
            .frame()
            .read_value::<1>(operand_types, rhs, &context.vm);

        let value = lhs.strict_equals(&rhs);

        context.vm.stack[(rp + output) as usize] = JsValue::from(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for StrictEq {
    const NAME: &'static str = "StrictEq";
    const INSTRUCTION: &'static str = "INST - StrictEq";
    const COST: u8 = 2;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let output = u32::from(context.vm.read::<u8>());
        let lhs = context.vm.read::<u8>().into();
        let rhs = context.vm.read::<u8>().into();
        Self::operation(output, lhs, rhs, operand_types, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let output = u32::from(context.vm.read::<u16>());
        let lhs = context.vm.read::<u16>().into();
        let rhs = context.vm.read::<u16>().into();
        Self::operation(output, lhs, rhs, operand_types, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let output = context.vm.read::<u32>();
        let lhs = context.vm.read::<u32>();
        let rhs = context.vm.read::<u32>();
        Self::operation(output, lhs, rhs, operand_types, context)
    }
}

/// `StrictNotEq` implements the Opcode Operation for `Opcode::StrictNotEq`
///
/// Operation:
///  - Binary `!==` operation
#[derive(Debug, Clone, Copy)]
pub(crate) struct StrictNotEq;

impl StrictNotEq {
    #[allow(clippy::unnecessary_wraps)]
    #[allow(clippy::needless_pass_by_value)]
    fn operation(
        output: u32,
        lhs: u32,
        rhs: u32,
        operand_types: u8,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let rp = context.vm.frame().rp;

        let lhs = context
            .vm
            .frame()
            .read_value::<0>(operand_types, lhs, &context.vm);
        let rhs = context
            .vm
            .frame()
            .read_value::<1>(operand_types, rhs, &context.vm);

        let value = !lhs.strict_equals(&rhs);

        context.vm.stack[(rp + output) as usize] = JsValue::from(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for StrictNotEq {
    const NAME: &'static str = "StrictNotEq";
    const INSTRUCTION: &'static str = "INST - StrictNotEq";
    const COST: u8 = 2;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let output = u32::from(context.vm.read::<u8>());
        let lhs = context.vm.read::<u8>().into();
        let rhs = context.vm.read::<u8>().into();
        Self::operation(output, lhs, rhs, operand_types, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let output = u32::from(context.vm.read::<u16>());
        let lhs = context.vm.read::<u16>().into();
        let rhs = context.vm.read::<u16>().into();
        Self::operation(output, lhs, rhs, operand_types, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let output = context.vm.read::<u32>();
        let lhs = context.vm.read::<u32>();
        let rhs = context.vm.read::<u32>();
        Self::operation(output, lhs, rhs, operand_types, context)
    }
}

/// `In` implements the Opcode Operation for `Opcode::In`
///
/// Operation:
///  - Binary `in` operation
#[derive(Debug, Clone, Copy)]
pub(crate) struct In;

impl In {
    #[allow(clippy::needless_pass_by_value)]
    fn operation(
        output: u32,
        lhs: u32,
        rhs: u32,
        operand_types: u8,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let rp = context.vm.frame().rp;
        let rhs = context
            .vm
            .frame()
            .read_value::<1>(operand_types, rhs, &context.vm);

        let Some(rhs) = rhs.as_object() else {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "right-hand side of 'in' should be an object, got `{}`",
                    rhs.type_of()
                ))
                .into());
        };

        let lhs = context
            .vm
            .frame()
            .read_value::<0>(operand_types, lhs, &context.vm);
        let key = lhs.to_property_key(context)?;
        let value = rhs.has_property(key, context)?;
        context.vm.stack[(rp + output) as usize] = JsValue::from(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for In {
    const NAME: &'static str = "In";
    const INSTRUCTION: &'static str = "INST - In";
    const COST: u8 = 3;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let output = u32::from(context.vm.read::<u8>());
        let lhs = context.vm.read::<u8>().into();
        let rhs = context.vm.read::<u8>().into();
        Self::operation(output, lhs, rhs, operand_types, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let output = u32::from(context.vm.read::<u16>());
        let lhs = context.vm.read::<u16>().into();
        let rhs = context.vm.read::<u16>().into();
        Self::operation(output, lhs, rhs, operand_types, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let output = context.vm.read::<u32>();
        let lhs = context.vm.read::<u32>();
        let rhs = context.vm.read::<u32>();
        Self::operation(output, lhs, rhs, operand_types, context)
    }
}

/// `InPrivate` implements the Opcode Operation for `Opcode::InPrivate`
///
/// Operation:
///  - Binary `in` operation for private names.
#[derive(Debug, Clone, Copy)]
pub(crate) struct InPrivate;

impl InPrivate {
    #[allow(clippy::needless_pass_by_value)]
    fn operation(
        dst: u32,
        index: usize,
        rhs: u32,
        operand_types: u8,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let name = context.vm.frame().code_block().constant_string(index);
        let rp = context.vm.frame().rp;
        let rhs = context
            .vm
            .frame()
            .read_value::<0>(operand_types, rhs, &context.vm);

        let Some(rhs) = rhs.as_object() else {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "right-hand side of 'in' should be an object, got `{}`",
                    rhs.type_of()
                ))
                .into());
        };

        let name = context
            .vm
            .environments
            .resolve_private_identifier(name)
            .expect("private name must be in environment");

        let value = rhs.private_element_find(&name, true, true).is_some();

        context.vm.stack[(rp + dst) as usize] = JsValue::from(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for InPrivate {
    const NAME: &'static str = "InPrivate";
    const INSTRUCTION: &'static str = "INST - InPrivate";
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let dst = u32::from(context.vm.read::<u8>());
        let index = context.vm.read::<u8>() as usize;
        let rhs = context.vm.read::<u8>().into();
        Self::operation(dst, index, rhs, operand_types, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let dst = u32::from(context.vm.read::<u16>());
        let index = context.vm.read::<u16>() as usize;
        let rhs = context.vm.read::<u16>().into();
        Self::operation(dst, index, rhs, operand_types, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let operand_types = context.vm.read::<u8>();
        let dst = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        let rhs = context.vm.read::<u32>();
        Self::operation(dst, index, rhs, operand_types, context)
    }
}
