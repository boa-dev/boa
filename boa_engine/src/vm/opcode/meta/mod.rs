use std::unreachable;

use crate::{
    module::ModuleKind,
    vm::{opcode::Operation, ActiveRunnable, CompletionType},
    Context, JsObject, JsResult, JsValue,
};

/// `NewTarget` implements the Opcode Operation for `Opcode::NewTarget`
///
/// Operation:
///  - Push the current new target to the stack.
#[derive(Debug, Clone, Copy)]
pub(crate) struct NewTarget;

impl Operation for NewTarget {
    const NAME: &'static str = "NewTarget";
    const INSTRUCTION: &'static str = "INST - NewTarget";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
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
        context.vm.push(new_target);
        Ok(CompletionType::Normal)
    }
}

/// `ImportMeta` implements the Opcode Operation for `Opcode::ImportMeta`
///
/// Operation:
///  - Push the current `import.meta` to the stack
///
/// Operands:
///
/// Stack:
///  - **=>** `import.meta`
#[derive(Debug, Clone, Copy)]
pub(crate) struct ImportMeta;

impl Operation for ImportMeta {
    const NAME: &'static str = "ImportMeta";
    const INSTRUCTION: &'static str = "INST - ImportMeta";

    fn execute(context: &mut Context<'_>) -> JsResult<CompletionType> {
        // 1. Let module be GetActiveScriptOrModule().

        let Some(ActiveRunnable::Module(module)) = context.vm.active_runnable.clone() else {
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
                //     a. Set importMeta to OrdinaryObjectCreate(null).
                let import_meta = JsObject::with_null_proto();

                //     b. Let importMetaValues be HostGetImportMetaProperties(module).
                //     c. For each Record { [[Key]], [[Value]] } p of importMetaValues, do
                //         i. Perform ! CreateDataPropertyOrThrow(importMeta, p.[[Key]], p.[[Value]]).
                //     d. Perform HostFinalizeImportMeta(importMeta, module).
                context
                    .module_loader()
                    .init_import_meta(&import_meta, &module, context);

                //     e. Set module.[[ImportMeta]] to importMeta.
                import_meta
            })
            .clone();

        //     b. Return importMeta.
        //     f. Return importMeta.
        context.vm.push(import_meta);

        Ok(CompletionType::Normal)
    }
}
