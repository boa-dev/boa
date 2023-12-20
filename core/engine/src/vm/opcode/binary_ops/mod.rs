use crate::{
    error::JsNativeError,
    vm::{opcode::Operation, CompletionType},
    Context, JsResult, JsValue,
};

pub(crate) mod logical;
pub(crate) mod macro_defined;

pub(crate) use logical::*;
pub(crate) use macro_defined::*;

use super::InstructionOperand;

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
        lhs: InstructionOperand,
        rhs: InstructionOperand,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let rp = context.vm.frame().rp;

        let lhs = lhs.to_value(&context.vm);
        let rhs = rhs.to_value(&context.vm);

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
        let output = u32::from(context.vm.read::<u8>());
        let lhs = InstructionOperand::from(context.vm.read::<u8>());
        let rhs = InstructionOperand::from(context.vm.read::<u8>());
        Self::operation(output, lhs, rhs, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let output = u32::from(context.vm.read::<u16>());
        let lhs = InstructionOperand::from(context.vm.read::<u16>());
        let rhs = InstructionOperand::from(context.vm.read::<u16>());
        Self::operation(output, lhs, rhs, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let output = context.vm.read::<u32>();
        let lhs = InstructionOperand::from(context.vm.read::<u32>());
        let rhs = InstructionOperand::from(context.vm.read::<u32>());
        Self::operation(output, lhs, rhs, context)
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
        lhs: InstructionOperand,
        rhs: InstructionOperand,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let rp = context.vm.frame().rp;

        let lhs = lhs.to_value(&context.vm);
        let rhs = rhs.to_value(&context.vm);

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
        let output = u32::from(context.vm.read::<u8>());
        let lhs = InstructionOperand::from(context.vm.read::<u8>());
        let rhs = InstructionOperand::from(context.vm.read::<u8>());
        Self::operation(output, lhs, rhs, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let output = u32::from(context.vm.read::<u16>());
        let lhs = InstructionOperand::from(context.vm.read::<u16>());
        let rhs = InstructionOperand::from(context.vm.read::<u16>());
        Self::operation(output, lhs, rhs, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let output = context.vm.read::<u32>();
        let lhs = InstructionOperand::from(context.vm.read::<u32>());
        let rhs = InstructionOperand::from(context.vm.read::<u32>());
        Self::operation(output, lhs, rhs, context)
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
        lhs: InstructionOperand,
        rhs: InstructionOperand,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let rp = context.vm.frame().rp;

        let lhs = lhs.to_value(&context.vm);
        let rhs = rhs.to_value(&context.vm);

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
        let output = u32::from(context.vm.read::<u8>());
        let lhs = InstructionOperand::from(context.vm.read::<u8>());
        let rhs = InstructionOperand::from(context.vm.read::<u8>());
        Self::operation(output, lhs, rhs, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let output = u32::from(context.vm.read::<u16>());
        let lhs = InstructionOperand::from(context.vm.read::<u16>());
        let rhs = InstructionOperand::from(context.vm.read::<u16>());
        Self::operation(output, lhs, rhs, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let output = context.vm.read::<u32>();
        let lhs = InstructionOperand::from(context.vm.read::<u32>());
        let rhs = InstructionOperand::from(context.vm.read::<u32>());
        Self::operation(output, lhs, rhs, context)
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
        lhs: InstructionOperand,
        rhs: InstructionOperand,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let rp = context.vm.frame().rp;
        let rhs = rhs.to_value(&context.vm);

        let Some(rhs) = rhs.as_object() else {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "right-hand side of 'in' should be an object, got `{}`",
                    rhs.type_of()
                ))
                .into());
        };

        let lhs = lhs.to_value(&context.vm);
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
        let output = u32::from(context.vm.read::<u8>());
        let lhs = InstructionOperand::from(context.vm.read::<u8>());
        let rhs = InstructionOperand::from(context.vm.read::<u8>());
        Self::operation(output, lhs, rhs, context)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let output = u32::from(context.vm.read::<u16>());
        let lhs = InstructionOperand::from(context.vm.read::<u16>());
        let rhs = InstructionOperand::from(context.vm.read::<u16>());
        Self::operation(output, lhs, rhs, context)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let output = context.vm.read::<u32>();
        let lhs = InstructionOperand::from(context.vm.read::<u32>());
        let rhs = InstructionOperand::from(context.vm.read::<u32>());
        Self::operation(output, lhs, rhs, context)
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
        context: &mut Context,
        dst: u32,
        index: usize,
        rhs: InstructionOperand,
    ) -> JsResult<CompletionType> {
        let name = context.vm.frame().code_block().constant_string(index);
        let rhs = rhs.to_value(&context.vm);

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

        let rp = context.vm.frame().rp;
        context.vm.stack[(rp + dst) as usize] = JsValue::from(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for InPrivate {
    const NAME: &'static str = "InPrivate";
    const INSTRUCTION: &'static str = "INST - InPrivate";
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u8>());
        let index = context.vm.read::<u8>() as usize;
        let rhs = InstructionOperand::from(context.vm.read::<u8>());
        Self::operation(context, dst, index, rhs)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let dst = u32::from(context.vm.read::<u16>());
        let index = context.vm.read::<u16>() as usize;
        let rhs = InstructionOperand::from(context.vm.read::<u16>());
        Self::operation(context, dst, index, rhs)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let dst = context.vm.read::<u32>();
        let index = context.vm.read::<u32>() as usize;
        let rhs = InstructionOperand::from(context.vm.read::<u32>());
        Self::operation(context, dst, index, rhs)
    }
}
