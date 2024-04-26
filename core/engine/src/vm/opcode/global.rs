use crate::{vm::CompletionType, Context, JsResult};

use super::Operation;

/// `HasRestrictedGlobalProperty` implements the Opcode Operation for `Opcode::HasRestrictedGlobalProperty`
///
/// Operation:
///  - Performs [`HasRestrictedGlobalProperty ( N )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-hasrestrictedglobalproperty
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
///  - Performs [`CanDeclareGlobalFunction ( N )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-candeclareglobalfunction
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
///  - Performs [`CanDeclareGlobalVar ( N )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-candeclareglobalvar
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

/// `CreateGlobalFunctionBinding` implements the Opcode Operation for `Opcode::CreateGlobalFunctionBinding`
///
/// Operation:
/// - Performs [`CreateGlobalFunctionBinding ( N, V, D )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-createglobalfunctionbinding
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateGlobalFunctionBinding;

impl CreateGlobalFunctionBinding {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        context: &mut Context,
        index: usize,
        configurable: bool,
    ) -> JsResult<CompletionType> {
        let name = context.vm.frame().code_block().constant_string(index);
        let value = context.vm.pop();

        let function = value
            .as_object()
            .expect("valeu should be an function")
            .clone();
        context.create_global_function_binding(name, function, configurable)?;

        Ok(CompletionType::Normal)
    }
}

impl Operation for CreateGlobalFunctionBinding {
    const NAME: &'static str = "CreateGlobalFunctionBinding";
    const INSTRUCTION: &'static str = "INST - CreateGlobalFunctionBinding";
    const COST: u8 = 2;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let configurable = context.vm.read::<u8>() != 0;
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index, configurable)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let configurable = context.vm.read::<u8>() != 0;
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index, configurable)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let configurable = context.vm.read::<u8>() != 0;
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index, configurable)
    }
}

/// `CreateGlobalVarBinding` implements the Opcode Operation for `Opcode::CreateGlobalVarBinding`
///
/// Operation:
/// - Performs [`CreateGlobalVarBinding ( N, V, D )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-createglobalvarbinding
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateGlobalVarBinding;

impl CreateGlobalVarBinding {
    #[allow(clippy::unnecessary_wraps)]
    fn operation(
        context: &mut Context,
        index: usize,
        configurable: bool,
    ) -> JsResult<CompletionType> {
        let name = context.vm.frame().code_block().constant_string(index);
        context.create_global_var_binding(name, configurable)?;

        Ok(CompletionType::Normal)
    }
}

impl Operation for CreateGlobalVarBinding {
    const NAME: &'static str = "CreateGlobalVarBinding";
    const INSTRUCTION: &'static str = "INST - CreateGlobalVarBinding";
    const COST: u8 = 2;

    fn execute(context: &mut Context) -> JsResult<CompletionType> {
        let configurable = context.vm.read::<u8>() != 0;
        let index = context.vm.read::<u8>() as usize;
        Self::operation(context, index, configurable)
    }

    fn execute_with_u16_operands(context: &mut Context) -> JsResult<CompletionType> {
        let configurable = context.vm.read::<u8>() != 0;
        let index = context.vm.read::<u16>() as usize;
        Self::operation(context, index, configurable)
    }

    fn execute_with_u32_operands(context: &mut Context) -> JsResult<CompletionType> {
        let configurable = context.vm.read::<u8>() != 0;
        let index = context.vm.read::<u32>() as usize;
        Self::operation(context, index, configurable)
    }
}
