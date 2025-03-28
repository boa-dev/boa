use super::VaryingOperand;
use crate::{
    module::ModuleKind,
    vm::{opcode::Operation, ActiveRunnable, CompletionType, Registers},
    Context, JsObject, JsResult, JsValue,
};
use std::unreachable;

/// `NewTarget` implements the Opcode Operation for `Opcode::NewTarget`
///
/// Operation:
///  - Push the current new target to the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct NewTarget;

impl NewTarget {
    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn operation(
        dst: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        let new_target = if let Some(new_target) = context
            .vm
            .environments
            .get_this_environment()
            .as_function()
            .and_then(|env| env.slots().new_target().cloned())
        {
            new_target.into()
        } else {
            JsValue::undefined()
        };
        registers.set(dst.into(), new_target);
        Ok(CompletionType::Normal)
    }
}

impl Operation for NewTarget {
    const NAME: &'static str = "NewTarget";
    const INSTRUCTION: &'static str = "INST - NewTarget";
    const COST: u8 = 2;
}

/// `ImportMeta` implements the Opcode Operation for `Opcode::ImportMeta`
///
/// Operation:
///  - Push the current `import.meta` to the stack
#[derive(Debug, Clone, Copy)]
pub(crate) struct ImportMeta;

impl ImportMeta {
    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn operation(
        dst: VaryingOperand,
        registers: &mut Registers,
        context: &mut Context,
    ) -> JsResult<CompletionType> {
        // Meta Properties
        //
        // ImportMeta : import . meta
        //
        // https://tc39.es/ecma262/#sec-meta-properties

        // 1. Let module be GetActiveScriptOrModule().

        let Some(ActiveRunnable::Module(module)) = context.get_active_script_or_module() else {
            unreachable!("2. Assert: module is a Source Text Module Record.");
        };

        let ModuleKind::SourceText(src) = module.kind() else {
            unreachable!("2. Assert: module is a Source Text Module Record.");
        };

        // 3. Let importMeta be module.[[ImportMeta]].
        // 4. If importMeta is empty, then
        // 5. Else,
        //     a. Assert: importMeta is an Object.
        let import_meta = src
            .import_meta()
            .borrow_mut()
            .get_or_insert_with(|| {
                // a. Set importMeta to OrdinaryObjectCreate(null).
                let import_meta = JsObject::with_null_proto();

                // b. Let importMetaValues be HostGetImportMetaProperties(module).
                // c. For each Record { [[Key]], [[Value]] } p of importMetaValues, do
                //     i. Perform ! CreateDataPropertyOrThrow(importMeta, p.[[Key]], p.[[Value]]).
                // d. Perform HostFinalizeImportMeta(importMeta, module).
                context
                    .module_loader()
                    .init_import_meta(&import_meta, &module, context);

                // e. Set module.[[ImportMeta]] to importMeta.
                import_meta
            })
            .clone();

        //     b. Return importMeta.
        //     f. Return importMeta.
        registers.set(dst.into(), import_meta.into());

        Ok(CompletionType::Normal)
    }
}

impl Operation for ImportMeta {
    const NAME: &'static str = "ImportMeta";
    const INSTRUCTION: &'static str = "INST - ImportMeta";
    const COST: u8 = 6;
}
