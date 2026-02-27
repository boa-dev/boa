use super::{Operation, VaryingOperand};
use crate::{Context, JsResult};

/// `HasRestrictedGlobalProperty` implements the Opcode Operation for `Opcode::HasRestrictedGlobalProperty`
///
/// Operation:
///  - Performs [`HasRestrictedGlobalProperty ( N )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-hasrestrictedglobalproperty
#[derive(Debug, Clone, Copy)]
pub(crate) struct HasRestrictedGlobalProperty;

impl HasRestrictedGlobalProperty {
    #[inline(always)]
    pub(super) fn operation(
        (dst, index): (VaryingOperand, VaryingOperand),
        context: &Context,
    ) -> JsResult<()> {
        let name = &context
            .vm_mut()
            .frame()
            .code_block()
            .constant_string(index.into());
        let value = context.has_restricted_global_property(name)?;
        context.vm_mut().set_register(dst.into(), value.into());
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
#[derive(Debug, Clone, Copy)]
pub(crate) struct CanDeclareGlobalFunction;

impl CanDeclareGlobalFunction {
    #[inline(always)]
    pub(super) fn operation(
        (dst, index): (VaryingOperand, VaryingOperand),
        context: &Context,
    ) -> JsResult<()> {
        let name = &context
            .vm_mut()
            .frame()
            .code_block()
            .constant_string(index.into());
        let value = context.can_declare_global_function(name)?;
        context.vm_mut().set_register(dst.into(), value.into());
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
#[derive(Debug, Clone, Copy)]
pub(crate) struct CanDeclareGlobalVar;

impl CanDeclareGlobalVar {
    #[inline(always)]
    pub(super) fn operation(
        (dst, index): (VaryingOperand, VaryingOperand),
        context: &Context,
    ) -> JsResult<()> {
        let name = &context
            .vm_mut()
            .frame()
            .code_block()
            .constant_string(index.into());
        let value = context.can_declare_global_var(name)?;
        context.vm_mut().set_register(dst.into(), value.into());
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
/// - Performs [`CreateGlobalFunctionBinding ( N, V, D )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-createglobalfunctionbinding
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreateGlobalFunctionBinding;

impl CreateGlobalFunctionBinding {
    #[inline(always)]
    pub(super) fn operation(
        (function, configurable, index): (VaryingOperand, VaryingOperand, VaryingOperand),
        context: &Context,
    ) -> JsResult<()> {
        let configurable = u32::from(configurable) != 0;
        let vm = context.vm_mut();
        let value = vm.get_register(function.into()).clone();
        let name = vm
            .frame()
            .code_block()
            .constant_string(index.into());

        let function = value
            .as_object()
            .expect("value must be an function");
        context.create_global_function_binding(name, function, configurable)?;

        Ok(())
    }
}

impl Operation for CreateGlobalFunctionBinding {
    const NAME: &'static str = "CreateGlobalFunctionBinding";
    const INSTRUCTION: &'static str = "INST - CreateGlobalFunctionBinding";
    const COST: u8 = 2;
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
    #[inline(always)]
    pub(super) fn operation(
        (configurable, index): (VaryingOperand, VaryingOperand),
        context: &Context,
    ) -> JsResult<()> {
        let configurable = u32::from(configurable) != 0;
        let name = context
            .vm_mut()
            .frame()
            .code_block()
            .constant_string(index.into());
        context.create_global_var_binding(name, configurable)?;

        Ok(())
    }
}

impl Operation for CreateGlobalVarBinding {
    const NAME: &'static str = "CreateGlobalVarBinding";
    const INSTRUCTION: &'static str = "INST - CreateGlobalVarBinding";
    const COST: u8 = 2;
}
