use crate::{vm::CompletionType, Context, JsResult};

use super::Operation;

/// `HasRestrictedGlobalProperty` implements the Opcode Operation for `Opcode::HasRestrictedGlobalProperty`
///
/// Operation:
///  - TODO: doc
#[derive(Debug, Clone, Copy)]
pub(crate) struct HasRestrictedGlobalProperty;

impl HasRestrictedGlobalProperty {
    fn operation(context: &mut Context, index: usize) -> JsResult<CompletionType> {
        let name = &context.vm.frame().code_block().constant_string(index);
        let value = context.has_restricted_global_property(name)?;
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for HasRestrictedGlobalProperty {
    const NAME: &'static str = "HasRestrictedGlobalProperty";
    const INSTRUCTION: &'static str = "INST - HasRestrictedGlobalProperty";
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}

/// `CanDeclareGlobalFunction` implements the Opcode Operation for `Opcode::CanDeclareGlobalFunction`
///
/// Operation:
///  - TODO: doc
#[derive(Debug, Clone, Copy)]
pub(crate) struct CanDeclareGlobalFunction;

impl CanDeclareGlobalFunction {
    fn operation(context: &mut Context, index: usize) -> JsResult<CompletionType> {
        let name = &context.vm.frame().code_block().constant_string(index);
        let value = context.can_declare_global_function(name)?;
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for CanDeclareGlobalFunction {
    const NAME: &'static str = "CanDeclareGlobalFunction";
    const INSTRUCTION: &'static str = "INST - CanDeclareGlobalFunction";
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}

/// `CanDeclareGlobalVar` implements the Opcode Operation for `Opcode::CanDeclareGlobalVar`
///
/// Operation:
///  - TODO: doc
#[derive(Debug, Clone, Copy)]
pub(crate) struct CanDeclareGlobalVar;

impl CanDeclareGlobalVar {
    fn operation(context: &mut Context, index: usize) -> JsResult<CompletionType> {
        let name = &context.vm.frame().code_block().constant_string(index);
        let value = context.can_declare_global_var(name)?;
        context.vm.push(value);
        Ok(CompletionType::Normal)
    }
}

impl Operation for CanDeclareGlobalVar {
    const NAME: &'static str = "CanDeclareGlobalVar";
    const INSTRUCTION: &'static str = "INST - CanDeclareGlobalVar";
    const COST: u8 = 4;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index)
    }
}
