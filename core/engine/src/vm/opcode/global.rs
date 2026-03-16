use crate::{
    Context, JsResult,
    vm::opcode::{IndexOperand, Operation, RegisterOperand},
};

/// `HasRestrictedGlobalProperty` implements the Opcode Operation for `Opcode::HasRestrictedGlobalProperty`
///
/// Operation:
///  - Performs [`HasRestrictedGlobalProperty ( N )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-hasrestrictedglobalproperty
pub(crate) struct HasRestrictedGlobalProperty;

impl HasRestrictedGlobalProperty {
    pub(crate) fn operation(
        (dst, index): (RegisterOperand, IndexOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let code_block = context.vm.frame().code_block();
        let name = code_block.constant_string(index.into());
        let result = context.has_restricted_global_property(&name)?;
        context.vm.set_register(dst.into(), result.into());
        Ok(())
    }
}

impl Operation for HasRestrictedGlobalProperty {
    const NAME: &'static str = "HasRestrictedGlobalProperty";
    const INSTRUCTION: &'static str = "INST - HasRestrictedGlobalProperty";
    const COST: u8 = 4;
}

/// `CanDeclareGlobalFunction` implements the Opcode Operation for `Opcode::CanDeclareGlobalFunction`
///
/// Operation:
///  - Performs [`CanDeclareGlobalFunction ( N )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-candeclareglobalfunction
pub(crate) struct CanDeclareGlobalFunction;

impl CanDeclareGlobalFunction {
    pub(crate) fn operation(
        (dst, index): (RegisterOperand, IndexOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let code_block = context.vm.frame().code_block();
        let name = code_block.constant_string(index.into());
        let result = context.can_declare_global_function(&name)?;
        context.vm.set_register(dst.into(), result.into());
        Ok(())
    }
}

impl Operation for CanDeclareGlobalFunction {
    const NAME: &'static str = "CanDeclareGlobalFunction";
    const INSTRUCTION: &'static str = "INST - CanDeclareGlobalFunction";
    const COST: u8 = 4;
}

/// `CanDeclareGlobalVar` implements the Opcode Operation for `Opcode::CanDeclareGlobalVar`
///
/// Operation:
///  - Performs [`CanDeclareGlobalVar ( N )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-candeclareglobalvar
pub(crate) struct CanDeclareGlobalVar;

impl CanDeclareGlobalVar {
    pub(crate) fn operation(
        (dst, index): (RegisterOperand, IndexOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let code_block = context.vm.frame().code_block();
        let name = code_block.constant_string(index.into());
        let result = context.can_declare_global_var(&name)?;
        context.vm.set_register(dst.into(), result.into());
        Ok(())
    }
}

impl Operation for CanDeclareGlobalVar {
    const NAME: &'static str = "CanDeclareGlobalVar";
    const INSTRUCTION: &'static str = "INST - CanDeclareGlobalVar";
    const COST: u8 = 4;
}

/// `CreateGlobalFunctionBinding` implements the Opcode Operation for `Opcode::CreateGlobalFunctionBinding`
///
/// Operation:
///  - Performs [`CreateGlobalFunctionBinding ( N, V, D )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-createglobalfunctionbinding
pub(crate) struct CreateGlobalFunctionBinding;

impl CreateGlobalFunctionBinding {
    pub(crate) fn operation(
        (src, configurable, name_index): (RegisterOperand, IndexOperand, IndexOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let code_block = context.vm.frame().code_block();
        let name = code_block.constant_string(name_index.into());
        let value = context.vm.get_register(src.into()).clone();
        let configurable = u32::from(configurable) != 0;

        // Convert JsValue to JsObject
        let function = value
            .as_object()
            .ok_or_else(|| crate::JsNativeError::typ().with_message("value is not an object"))?
            .clone();

        context.create_global_function_binding(name, function, configurable)?;
        Ok(())
    }
}

impl Operation for CreateGlobalFunctionBinding {
    const NAME: &'static str = "CreateGlobalFunctionBinding";
    const INSTRUCTION: &'static str = "INST - CreateGlobalFunctionBinding";
    const COST: u8 = 4;
}

/// `CreateGlobalVarBinding` implements the Opcode Operation for `Opcode::CreateGlobalVarBinding`
///
/// Operation:
///  - Performs [`CreateGlobalVarBinding ( N, D )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-createglobalvarbinding
pub(crate) struct CreateGlobalVarBinding;

impl CreateGlobalVarBinding {
    pub(crate) fn operation(
        (configurable, name_index): (IndexOperand, IndexOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let code_block = context.vm.frame().code_block();
        let name = code_block.constant_string(name_index.into());
        let configurable = u32::from(configurable) != 0;
        context.create_global_var_binding(name, configurable)?;
        Ok(())
    }
}

impl Operation for CreateGlobalVarBinding {
    const NAME: &'static str = "CreateGlobalVarBinding";
    const INSTRUCTION: &'static str = "INST - CreateGlobalVarBinding";
    const COST: u8 = 4;
}
