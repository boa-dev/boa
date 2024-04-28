use std::{cell::Cell, collections::HashSet, hash::BuildHasherDefault, rc::Rc};

use boa_ast::{
    declaration::{
        ExportEntry, ImportEntry, ImportName, IndirectExportEntry, LexicalDeclaration,
        LocalExportEntry, ReExportImportName,
    },
    operations::{
        bound_names, contains, lexically_scoped_declarations, var_scoped_declarations,
        ContainsSymbol, LexicallyScopedDeclaration,
    },
};
use boa_gc::{Finalize, Gc, GcRefCell, Trace};
use boa_interner::Interner;
use boa_macros::js_str;
use indexmap::IndexSet;
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

use crate::{
    builtins::{promise::PromiseCapability, Promise},
    bytecompiler::{ByteCompiler, FunctionSpec, ToJsString},
    environments::{
        BindingLocator, CompileTimeEnvironment, DeclarativeEnvironment, EnvironmentStack,
    },
    js_string,
    module::ModuleKind,
    object::{FunctionObjectBuilder, JsPromise},
    realm::Realm,
    vm::{
        create_function_object_fast, ActiveRunnable, CallFrame, CallFrameFlags, CodeBlock,
        CodeBlockFlags, CompletionRecord, Opcode,
    },
    Context, JsArgs, JsError, JsNativeError, JsObject, JsResult, JsString, JsValue, NativeFunction,
};

use super::{
    BindingName, GraphLoadingState, Module, Referrer, ResolveExportError, ResolvedBinding,
};

/// Information for the [**Depth-first search**] algorithm used in the
/// [`Module::link`] and [`Module::evaluate`] methods.
#[derive(Clone, Copy, Debug, Trace, Finalize)]
#[boa_gc(empty_trace)]
pub(super) struct DfsInfo {
    dfs_index: usize,
    dfs_ancestor_index: usize,
}

/// Current status of a [`SourceTextModule`].
///
/// Roughly corresponds to the `[[Status]]` field of [**Cyclic Module Records**][cyclic],
/// but with a state machine-like design for better correctness.
///
/// [cyclic]: https://tc39.es/ecma262/#table-cyclic-module-fields
#[derive(Debug, Trace, Finalize, Default)]
#[boa_gc(unsafe_no_drop)]
enum ModuleStatus {
    #[default]
    Unlinked,
    Linking {
        info: DfsInfo,
    },
    PreLinked {
        environment: Gc<DeclarativeEnvironment>,
        context: SourceTextContext,
        info: DfsInfo,
    },
    Linked {
        environment: Gc<DeclarativeEnvironment>,
        context: SourceTextContext,
        info: DfsInfo,
    },
    Evaluating {
        environment: Gc<DeclarativeEnvironment>,
        context: SourceTextContext,
        top_level_capability: Option<PromiseCapability>,
        cycle_root: Module,
        info: DfsInfo,
        async_eval_index: Option<usize>,
    },
    EvaluatingAsync {
        environment: Gc<DeclarativeEnvironment>,
        context: SourceTextContext,
        top_level_capability: Option<PromiseCapability>,
        cycle_root: Module,
        async_eval_index: usize,
        pending_async_dependencies: usize,
    },
    Evaluated {
        environment: Gc<DeclarativeEnvironment>,
        top_level_capability: Option<PromiseCapability>,
        cycle_root: Module,
        error: Option<JsError>,
    },
}

impl ModuleStatus {
    /// Transition from one state to another, taking the current state by value to move data
    /// between states.
    fn transition<F>(&mut self, f: F)
    where
        F: FnOnce(Self) -> Self,
    {
        *self = f(std::mem::take(self));
    }

    /// Gets the current index info of the module within the dependency graph, or `None` if the
    /// module is not in a state executing the dfs algorithm.
    const fn dfs_info(&self) -> Option<&DfsInfo> {
        match self {
            Self::Unlinked | Self::EvaluatingAsync { .. } | Self::Evaluated { .. } => None,
            Self::Linking { info }
            | Self::PreLinked { info, .. }
            | Self::Linked { info, .. }
            | Self::Evaluating { info, .. } => Some(info),
        }
    }

    /// Gets a mutable reference to the current index info of the module within the dependency graph,
    /// or `None` if the module is not in a state executing the dfs algorithm.
    fn dfs_info_mut(&mut self) -> Option<&mut DfsInfo> {
        match self {
            Self::Unlinked | Self::EvaluatingAsync { .. } | Self::Evaluated { .. } => None,
            Self::Linking { info }
            | Self::PreLinked { info, .. }
            | Self::Linked { info, .. }
            | Self::Evaluating { info, .. } => Some(info),
        }
    }

    /// If this module is the top module being evaluated and is in the evaluating state, gets its top
    /// level capability.
    const fn top_level_capability(&self) -> Option<&PromiseCapability> {
        match &self {
            Self::Unlinked
            | Self::Linking { .. }
            | Self::PreLinked { .. }
            | Self::Linked { .. } => None,
            Self::Evaluating {
                top_level_capability,
                ..
            }
            | Self::EvaluatingAsync {
                top_level_capability,
                ..
            }
            | Self::Evaluated {
                top_level_capability,
                ..
            } => top_level_capability.as_ref(),
        }
    }

    /// If this module is in the evaluated state, gets its `error` field.
    const fn evaluation_error(&self) -> Option<&JsError> {
        match &self {
            Self::Evaluated { error, .. } => error.as_ref(),
            _ => None,
        }
    }

    /// If this module is in the evaluating state, gets its cycle root.
    const fn cycle_root(&self) -> Option<&Module> {
        match &self {
            Self::Evaluating { cycle_root, .. }
            | Self::EvaluatingAsync { cycle_root, .. }
            | Self::Evaluated { cycle_root, .. } => Some(cycle_root),
            _ => None,
        }
    }

    /// Gets the declarative environment from the module status.
    fn environment(&self) -> Option<Gc<DeclarativeEnvironment>> {
        match self {
            ModuleStatus::Unlinked | ModuleStatus::Linking { .. } => None,
            ModuleStatus::PreLinked { environment, .. }
            | ModuleStatus::Linked { environment, .. }
            | ModuleStatus::Evaluating { environment, .. }
            | ModuleStatus::EvaluatingAsync { environment, .. }
            | ModuleStatus::Evaluated { environment, .. } => Some(environment.clone()),
        }
    }
}

/// The execution context of a [`SourceTextModule`].
///
/// Stores the required context data that needs to be in place before executing the
/// inner code of the module.
#[derive(Clone, Trace, Finalize)]
#[boa_gc(unsafe_no_drop)]
struct SourceTextContext {
    codeblock: Gc<CodeBlock>,
    environments: EnvironmentStack,
    realm: Realm,
}

impl std::fmt::Debug for SourceTextContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceTextContext")
            .field("codeblock", &self.codeblock)
            .field("environments", &self.environments)
            .field("realm", &self.realm.addr())
            .finish()
    }
}

/// ECMAScript's [**Source Text Module Records**][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-source-text-module-records
#[derive(Trace, Finalize)]
pub(crate) struct SourceTextModule {
    status: GcRefCell<ModuleStatus>,
    loaded_modules: GcRefCell<FxHashMap<JsString, Module>>,
    async_parent_modules: GcRefCell<Vec<Module>>,
    import_meta: GcRefCell<Option<JsObject>>,
    #[unsafe_ignore_trace]
    code: ModuleCode,
}

impl std::fmt::Debug for SourceTextModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceTextModule")
            .field("status", &self.status)
            .field("loaded_modules", &self.loaded_modules)
            .field("async_parent_modules", &self.async_parent_modules)
            .field("import_meta", &self.import_meta)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct ModuleCode {
    has_tla: bool,
    requested_modules: IndexSet<JsString, BuildHasherDefault<FxHasher>>,
    source: boa_ast::Module,
    import_entries: Vec<ImportEntry>,
    local_export_entries: Vec<LocalExportEntry>,
    indirect_export_entries: Vec<IndirectExportEntry>,
    star_export_entries: Vec<JsString>,
}

impl SourceTextModule {
    /// Creates a new `SourceTextModule` from a parsed `ModuleSource`.
    ///
    /// Contains part of the abstract operation [`ParseModule`][parse].
    ///
    /// [parse]: https://tc39.es/ecma262/#sec-parsemodule
    pub(super) fn new(code: boa_ast::Module, interner: &Interner) -> Self {
        // 3. Let requestedModules be the ModuleRequests of body.
        let requested_modules = code
            .items()
            .requests()
            .iter()
            .map(|name| name.to_js_string(interner))
            .collect();
        // 4. Let importEntries be ImportEntries of body.
        let import_entries = code.items().import_entries();

        // 5. Let importedBoundNames be ImportedLocalNames(importEntries).
        // Can be ignored because this is just a simple `Iter::map`

        // 6. Let indirectExportEntries be a new empty List.
        let mut indirect_export_entries = Vec::new();
        // 7. Let localExportEntries be a new empty List.
        let mut local_export_entries = Vec::new();
        // 8. Let starExportEntries be a new empty List.
        let mut star_export_entries = Vec::new();

        // 10. For each ExportEntry Record ee of exportEntries, do
        for ee in code.items().export_entries() {
            match ee {
                // a. If ee.[[ModuleRequest]] is null, then
                ExportEntry::Ordinary(entry) => {
                    // ii. Else,
                    //     1. Let ie be the element of importEntries whose [[LocalName]] is ee.[[LocalName]].
                    if let Some((module, import)) =
                        import_entries.iter().find_map(|ie| match ie.import_name() {
                            ImportName::Name(name) if ie.local_name() == entry.local_name() => {
                                Some((ie.module_request(), name))
                            }
                            _ => None,
                        })
                    {
                        // 3. Else,
                        //    a. NOTE: This is a re-export of a single name.
                        //    b. Append the ExportEntry Record { [[ModuleRequest]]: ie.[[ModuleRequest]],
                        //       [[ImportName]]: ie.[[ImportName]], [[LocalName]]: null,
                        //       [[ExportName]]: ee.[[ExportName]] } to indirectExportEntries.
                        indirect_export_entries.push(IndirectExportEntry::new(
                            module,
                            ReExportImportName::Name(import),
                            entry.export_name(),
                        ));
                    } else {
                        // i. If importedBoundNames does not contain ee.[[LocalName]], then
                        //    1. Append ee to localExportEntries.

                        //    2. If ie.[[ImportName]] is namespace-object, then
                        //       a. NOTE: This is a re-export of an imported module namespace object.
                        //       b. Append ee to localExportEntries.
                        local_export_entries.push(entry);
                    }
                }
                // b. Else if ee.[[ImportName]] is all-but-default, then
                ExportEntry::StarReExport { module_request } => {
                    // i. Assert: ee.[[ExportName]] is null.
                    // ii. Append ee to starExportEntries.
                    star_export_entries.push(module_request.to_js_string(interner));
                }
                // c. Else,
                //    i. Append ee to indirectExportEntries.
                ExportEntry::ReExport(entry) => indirect_export_entries.push(entry),
            }
        }

        // 11. Let async be body Contains await.
        let has_tla = contains(&code, ContainsSymbol::AwaitExpression);

        // 12. Return Source Text Module Record {
        //     [[Realm]]: realm, [[Environment]]: empty, [[Namespace]]: empty, [[CycleRoot]]: empty,
        //     [[HasTLA]]: async, [[AsyncEvaluation]]: false, [[TopLevelCapability]]: empty,
        //     [[AsyncParentModules]]: « », [[PendingAsyncDependencies]]: empty,
        //     [[Status]]: new, [[EvaluationError]]: empty, [[HostDefined]]: hostDefined,
        //     [[ECMAScriptCode]]: body, [[Context]]: empty, [[ImportMeta]]: empty,
        //     [[RequestedModules]]: requestedModules, [[LoadedModules]]: « »,
        //     [[ImportEntries]]: importEntries, [[LocalExportEntries]]: localExportEntries,
        //     [[IndirectExportEntries]]: indirectExportEntries,
        //     [[StarExportEntries]]: starExportEntries,
        //     [[DFSIndex]]: empty, [[DFSAncestorIndex]]: empty
        // }.
        // Most of this can be ignored, since `Status` takes care of the remaining state.
        Self {
            status: GcRefCell::default(),
            loaded_modules: GcRefCell::default(),
            async_parent_modules: GcRefCell::default(),
            import_meta: GcRefCell::default(),
            code: ModuleCode {
                source: code,
                requested_modules,
                has_tla,
                import_entries,
                local_export_entries,
                indirect_export_entries,
                star_export_entries,
            },
        }
    }

    /// Abstract operation [`InnerModuleLoading`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-InnerModuleLoading
    pub(super) fn inner_load(
        &self,
        module_self: &Module,
        state: &Rc<GraphLoadingState>,
        context: &mut Context,
    ) {
        // 2. If module is a Cyclic Module Record, module.[[Status]] is new, and state.[[Visited]] does not contain
        //    module, then
        // a. Append module to state.[[Visited]].
        if matches!(&*self.status.borrow(), ModuleStatus::Unlinked)
            && state.visited.borrow_mut().insert(module_self.clone())
        {
            // b. Let requestedModulesCount be the number of elements in module.[[RequestedModules]].
            let requested = &self.code.requested_modules;
            // c. Set state.[[PendingModulesCount]] to state.[[PendingModulesCount]] + requestedModulesCount.
            state
                .pending_modules
                .set(state.pending_modules.get() + requested.len());
            // d. For each String required of module.[[RequestedModules]], do
            for required in requested.iter().cloned() {
                // i. If module.[[LoadedModules]] contains a Record whose [[Specifier]] is required, then
                let loaded = self.loaded_modules.borrow().get(&required).cloned();
                if let Some(loaded) = loaded {
                    // 1. Let record be that Record.
                    // 2. Perform InnerModuleLoading(state, record.[[Module]]).
                    loaded.inner_load(state, context);
                } else {
                    //    ii. Else,
                    //       1. Perform HostLoadImportedModule(module, required, state.[[HostDefined]], state).
                    //       2. NOTE: HostLoadImportedModule will call FinishLoadingImportedModule, which re-enters
                    //          the graph loading process through ContinueModuleLoading.
                    let name_specifier = required.clone();
                    let src = module_self.clone();
                    let state = state.clone();
                    context.module_loader().load_imported_module(
                        Referrer::Module(module_self.clone()),
                        name_specifier,
                        Box::new(move |completion, context| {
                            // FinishLoadingImportedModule ( referrer, specifier, payload, result )
                            // https://tc39.es/ecma262/#sec-FinishLoadingImportedModule

                            // 1. If result is a normal completion, then
                            if let Ok(loaded) = &completion {
                                let ModuleKind::SourceText(src) = src.kind() else {
                                    unreachable!("captured src must be a source text module");
                                };
                                // a. If referrer.[[LoadedModules]] contains a Record whose [[Specifier]] is specifier, then
                                // b. Else,
                                //    i. Append the Record { [[Specifier]]: specifier, [[Module]]: result.[[Value]] } to referrer.[[LoadedModules]].
                                let mut loaded_modules = src.loaded_modules.borrow_mut();
                                let entry = loaded_modules
                                    .entry(required)
                                    .or_insert_with(|| loaded.clone());

                                //    i. Assert: That Record's [[Module]] is result.[[Value]].
                                assert_eq!(entry, loaded);
                            }

                            // 2. If payload is a GraphLoadingState Record, then
                            //    a. Perform ContinueModuleLoading(payload, result).

                            // Abstract operation `ContinueModuleLoading ( state, moduleCompletion )`.
                            //
                            // https://tc39.es/ecma262/#sec-ContinueModuleLoading

                            // 1. If state.[[IsLoading]] is false, return unused.
                            if !state.loading.get() {
                                return;
                            }

                            // 2. If moduleCompletion is a normal completion, then
                            match completion {
                                Ok(m) => {
                                    // a. Perform InnerModuleLoading(state, moduleCompletion.[[Value]]).
                                    m.inner_load(&state, context);
                                }
                                // 3. Else,
                                Err(err) => {
                                    // a. Set state.[[IsLoading]] to false.
                                    state.loading.set(false);

                                    // b. Perform ! Call(state.[[PromiseCapability]].[[Reject]], undefined, « moduleCompletion.[[Value]] »).
                                    state
                                        .capability
                                        .reject()
                                        .call(
                                            &JsValue::undefined(),
                                            &[err.to_opaque(context)],
                                            context,
                                        )
                                        .expect("cannot fail for the default reject function");
                                }
                            }

                            // 4. Return unused.
                        }),
                        context,
                    );
                }
                // iii. If state.[[IsLoading]] is false, return unused.
                if !state.loading.get() {
                    return;
                }
            }
        }
    }

    /// Concrete method [`GetExportedNames ( [ exportStarSet ] )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getexportednames
    pub(super) fn get_exported_names(
        &self,
        module_self: &Module,
        export_star_set: &mut Vec<Module>,
        interner: &Interner,
    ) -> FxHashSet<JsString> {
        // 1. Assert: module.[[Status]] is not new.
        // 2. If exportStarSet is not present, set exportStarSet to a new empty List.

        // 3. If exportStarSet contains module, then
        if export_star_set.contains(module_self) {
            // a. Assert: We've reached the starting point of an export * circularity.
            // b. Return a new empty List.
            return FxHashSet::default();
        }

        // 4. Append module to exportStarSet.
        export_star_set.push(module_self.clone());

        // 5. Let exportedNames be a new empty List.
        let mut exported_names = FxHashSet::default();

        // 6. For each ExportEntry Record e of module.[[LocalExportEntries]], do
        for e in &self.code.local_export_entries {
            // a. Assert: module provides the direct binding for this export.
            // b. Append e.[[ExportName]] to exportedNames.
            let name = e.export_name().to_js_string(interner);
            exported_names.insert(name);
        }

        // 7. For each ExportEntry Record e of module.[[IndirectExportEntries]], do
        for e in &self.code.indirect_export_entries {
            // a. Assert: module imports a specific binding for this export.
            // b. Append e.[[ExportName]] to exportedNames.
            let name = e.export_name().to_js_string(interner);
            exported_names.insert(name);
        }

        // 8. For each ExportEntry Record e of module.[[StarExportEntries]], do
        for e in &self.code.star_export_entries {
            // a. Let requestedModule be GetImportedModule(module, e.[[ModuleRequest]]).
            let requested_module = self.loaded_modules.borrow()[e].clone();

            // b. Let starNames be requestedModule.GetExportedNames(exportStarSet).
            // c. For each element n of starNames, do
            for n in requested_module.get_exported_names(export_star_set, interner) {
                // i. If SameValue(n, "default") is false, then
                if n != js_str!("default") {
                    // 1. If exportedNames does not contain n, then
                    //    a. Append n to exportedNames.
                    exported_names.insert(n);
                }
            }
        }

        // 9. Return exportedNames.
        exported_names
    }

    /// Concrete method [`ResolveExport ( exportName [ , resolveSet ] )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-resolveexport
    #[allow(clippy::mutable_key_type)]
    pub(super) fn resolve_export(
        &self,
        module_self: &Module,
        export_name: &JsString,
        resolve_set: &mut FxHashSet<(Module, JsString)>,
        interner: &Interner,
    ) -> Result<ResolvedBinding, ResolveExportError> {
        // 1. Assert: module.[[Status]] is not new.
        // 2. If resolveSet is not present, set resolveSet to a new empty List.
        // 3. For each Record { [[Module]], [[ExportName]] } r of resolveSet, do
        //    a. If module and r.[[Module]] are the same Module Record and SameValue(exportName, r.[[ExportName]]) is true, then
        let value = (module_self.clone(), export_name.clone());
        if resolve_set.contains(&value) {
            //   i. Assert: This is a circular import request.
            //   ii. Return null.
            return Err(ResolveExportError::NotFound);
        }

        // 4. Append the Record { [[Module]]: module, [[ExportName]]: exportName } to resolveSet.
        resolve_set.insert(value);

        // 5. For each ExportEntry Record e of module.[[LocalExportEntries]], do
        for e in &self.code.local_export_entries {
            // a. If SameValue(exportName, e.[[ExportName]]) is true, then
            if export_name == &e.export_name().to_js_string(interner) {
                // i. Assert: module provides the direct binding for this export.
                // ii. Return ResolvedBinding Record { [[Module]]: module, [[BindingName]]: e.[[LocalName]] }.
                return Ok(ResolvedBinding {
                    module: module_self.clone(),
                    binding_name: BindingName::Name(e.local_name().to_js_string(interner)),
                });
            }
        }

        // 6. For each ExportEntry Record e of module.[[IndirectExportEntries]], do
        for e in &self.code.indirect_export_entries {
            // a. If SameValue(exportName, e.[[ExportName]]) is true, then
            if export_name == &e.export_name().to_js_string(interner) {
                // i. Let importedModule be GetImportedModule(module, e.[[ModuleRequest]]).
                let module_request = e.module_request().to_js_string(interner);
                let imported_module = self.loaded_modules.borrow()[&module_request].clone();
                return match e.import_name() {
                    // ii. If e.[[ImportName]] is all, then
                    //    1. Assert: module does not provide the direct binding for this export.
                    //    2. Return ResolvedBinding Record { [[Module]]: importedModule, [[BindingName]]: namespace }.
                    ReExportImportName::Star => Ok(ResolvedBinding {
                        module: imported_module,
                        binding_name: BindingName::Namespace,
                    }),
                    // iii. Else,
                    //    1. Assert: module imports a specific binding for this export.
                    //    2. Return importedModule.ResolveExport(e.[[ImportName]], resolveSet).
                    ReExportImportName::Name(name) => {
                        let name = name.to_js_string(interner);
                        imported_module.resolve_export(name, resolve_set, interner)
                    }
                };
            }
        }

        // 7. If SameValue(exportName, "default") is true, then
        if export_name == &js_str!("default") {
            // a. Assert: A default export was not explicitly defined by this module.
            // b. Return null.
            // c. NOTE: A default export cannot be provided by an export * from "mod" declaration.
            return Err(ResolveExportError::NotFound);
        }

        // 8. Let starResolution be null.
        let mut star_resolution: Option<ResolvedBinding> = None;

        // 9. For each ExportEntry Record e of module.[[StarExportEntries]], do
        for e in &self.code.star_export_entries {
            // a. Let importedModule be GetImportedModule(module, e.[[ModuleRequest]]).
            let imported_module = self.loaded_modules.borrow()[e].clone();
            // b. Let resolution be importedModule.ResolveExport(exportName, resolveSet).
            let resolution =
                match imported_module.resolve_export(export_name.clone(), resolve_set, interner) {
                    // d. If resolution is not null, then
                    Ok(resolution) => resolution,
                    // c. If resolution is ambiguous, return ambiguous.
                    Err(e @ ResolveExportError::Ambiguous) => return Err(e),
                    Err(ResolveExportError::NotFound) => continue,
                };

            // i. Assert: resolution is a ResolvedBinding Record.
            if let Some(star_resolution) = &star_resolution {
                // iii. Else,
                //    1. Assert: There is more than one * import that includes the requested name.
                //    2. If resolution.[[Module]] and starResolution.[[Module]] are not the same Module Record,
                //       return ambiguous.
                if resolution.module != star_resolution.module {
                    return Err(ResolveExportError::Ambiguous);
                }
                match (
                    resolution.binding_name,
                    star_resolution.binding_name.clone(),
                ) {
                    // 3. If resolution.[[BindingName]] is not starResolution.[[BindingName]] and either
                    //    resolution.[[BindingName]] or starResolution.[[BindingName]] is namespace,
                    //    return ambiguous.
                    (BindingName::Namespace, BindingName::Name(_))
                    | (BindingName::Name(_), BindingName::Namespace) => {
                        return Err(ResolveExportError::Ambiguous);
                    }
                    // 4. If resolution.[[BindingName]] is a String, starResolution.[[BindingName]] is a
                    //    String, and SameValue(resolution.[[BindingName]], starResolution.[[BindingName]])
                    //    is false, return ambiguous.
                    (BindingName::Name(res), BindingName::Name(star)) if res != star => {
                        return Err(ResolveExportError::Ambiguous);
                    }
                    _ => {}
                }
            } else {
                // ii. If starResolution is null, then
                //    1. Set starResolution to resolution.
                star_resolution = Some(resolution);
            }
        }

        // 10. Return starResolution.
        star_resolution.ok_or(ResolveExportError::NotFound)
    }

    /// Concrete method [`Link ( )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-moduledeclarationlinking
    pub(super) fn link(&self, module_self: &Module, context: &mut Context) -> JsResult<()> {
        // 1. Assert: module.[[Status]] is one of unlinked, linked, evaluating-async, or evaluated.
        debug_assert!(matches!(
            &*self.status.borrow(),
            ModuleStatus::Unlinked
                | ModuleStatus::Linked { .. }
                | ModuleStatus::EvaluatingAsync { .. }
                | ModuleStatus::Evaluated { .. }
        ));

        // 2. Let stack be a new empty List.
        let mut stack = Vec::new();

        // 3. Let result be Completion(InnerModuleLinking(module, stack, 0)).
        // 4. If result is an abrupt completion, then
        if let Err(err) = self.inner_link(module_self, &mut stack, 0, context) {
            // a. For each Cyclic Module Record m of stack, do
            for m in stack.iter().filter_map(|cmr| cmr.kind().as_source_text()) {
                // i. Assert: m.[[Status]] is linking.
                debug_assert!(matches!(&*m.status.borrow(), ModuleStatus::Linking { .. }));
                // ii. Set m.[[Status]] to unlinked.
                *m.status.borrow_mut() = ModuleStatus::Unlinked;
            }
            // b. Assert: module.[[Status]] is unlinked.
            debug_assert!(matches!(&*self.status.borrow(), ModuleStatus::Unlinked));
            // c. Return ? result.
            return Err(err);
        }

        // 5. Assert: module.[[Status]] is one of linked, evaluating-async, or evaluated.
        debug_assert!(matches!(
            &*self.status.borrow(),
            ModuleStatus::Linked { .. }
                | ModuleStatus::EvaluatingAsync { .. }
                | ModuleStatus::Evaluated { .. }
        ));
        // 6. Assert: stack is empty.
        assert!(stack.is_empty());

        // 7. Return unused.
        Ok(())
    }

    /// Abstract operation [`InnerModuleLinking ( module, stack, index )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-InnerModuleLinking
    pub(super) fn inner_link(
        &self,
        module_self: &Module,
        stack: &mut Vec<Module>,
        mut index: usize,
        context: &mut Context,
    ) -> JsResult<usize> {
        // 2. If module.[[Status]] is one of linking, linked, evaluating-async, or evaluated, then
        if matches!(
            &*self.status.borrow(),
            ModuleStatus::Linking { .. }
                | ModuleStatus::PreLinked { .. }
                | ModuleStatus::Linked { .. }
                | ModuleStatus::EvaluatingAsync { .. }
                | ModuleStatus::Evaluated { .. }
        ) {
            // a. Return index.
            return Ok(index);
        }

        // 3. Assert: module.[[Status]] is unlinked.
        debug_assert!(matches!(&*self.status.borrow(), ModuleStatus::Unlinked));

        // 4. Set module.[[Status]] to linking.
        // 5. Set module.[[DFSIndex]] to index.
        // 6. Set module.[[DFSAncestorIndex]] to index.
        *self.status.borrow_mut() = ModuleStatus::Linking {
            info: DfsInfo {
                dfs_index: index,
                dfs_ancestor_index: index,
            },
        };

        // 7. Set index to index + 1.
        index += 1;

        // 8. Append module to stack.
        stack.push(module_self.clone());

        // 9. For each String required of module.[[RequestedModules]], do

        for required in &self.code.requested_modules {
            // a. Let requiredModule be GetImportedModule(module, required).
            let required_module = self.loaded_modules.borrow()[required].clone();

            // b. Set index to ? InnerModuleLinking(requiredModule, stack, index).
            index = required_module.inner_link(stack, index, context)?;
            // c. If requiredModule is a Cyclic Module Record, then
            if let ModuleKind::SourceText(required_module_src) = required_module.kind() {
                // i. Assert: requiredModule.[[Status]] is one of linking, linked, evaluating-async, or evaluated.
                // ii. Assert: requiredModule.[[Status]] is linking if and only if stack contains requiredModule.
                debug_assert!(match &*required_module_src.status.borrow() {
                    ModuleStatus::PreLinked { .. }
                    | ModuleStatus::Linked { .. }
                    | ModuleStatus::EvaluatingAsync { .. }
                    | ModuleStatus::Evaluated { .. } => true,
                    ModuleStatus::Linking { .. } if stack.contains(&required_module) => true,
                    _ => false,
                });

                // iii. If requiredModule.[[Status]] is linking, then
                let required_index = if let ModuleStatus::Linking {
                    info:
                        DfsInfo {
                            dfs_ancestor_index, ..
                        },
                } = &*required_module_src.status.borrow()
                {
                    // 1. Set module.[[DFSAncestorIndex]] to min(module.[[DFSAncestorIndex]],
                    //    requiredModule.[[DFSAncestorIndex]]).

                    Some(*dfs_ancestor_index)
                } else {
                    None
                };

                if let Some(required_index) = required_index {
                    let mut status = self.status.borrow_mut();

                    let DfsInfo {
                        dfs_ancestor_index, ..
                    } = status
                        .dfs_info_mut()
                        .expect("should be on the linking state");
                    *dfs_ancestor_index = usize::min(*dfs_ancestor_index, required_index);
                }
            }
        }

        // 10. Perform ? module.InitializeEnvironment().
        self.initialize_environment(module_self, context)?;

        // 11. Assert: module occurs exactly once in stack.
        debug_assert_eq!(
            stack.iter().filter(|module| *module == module_self).count(),
            1
        );
        // 12. Assert: module.[[DFSAncestorIndex]] ≤ module.[[DFSIndex]].
        debug_assert!({
            let DfsInfo {
                dfs_ancestor_index,
                dfs_index,
            } = self
                .status
                .borrow()
                .dfs_info()
                .copied()
                .expect("should be linking");
            dfs_ancestor_index <= dfs_index
        });

        let info = self.status.borrow().dfs_info().copied();
        match info {
            // 13. If module.[[DFSAncestorIndex]] = module.[[DFSIndex]], then

            //     a. Let done be false.
            //     b. Repeat, while done is false,
            Some(info) if info.dfs_ancestor_index == info.dfs_index => loop {
                //    i. Let requiredModule be the last element of stack.
                //    ii. Remove the last element of stack.
                let last = stack.pop().expect("should have at least one element");
                let ModuleKind::SourceText(last_src) = last.kind() else {
                    unreachable!("iii. Assert: requiredModule is a Cyclic Module Record.")
                };

                //    iv. Set requiredModule.[[Status]] to linked.
                last_src
                    .status
                    .borrow_mut()
                    .transition(|current| match current {
                        ModuleStatus::PreLinked {
                            info,
                            context,
                            environment,
                        } => ModuleStatus::Linked {
                            info,
                            context,
                            environment,
                        },
                        _ => {
                            unreachable!(
                                "can only transition to `Linked` from the `PreLinked` state"
                            )
                        }
                    });

                //    v. If requiredModule and module are the same Module Record, set done to true.
                if &last == module_self {
                    break;
                }
            },
            _ => {}
        }

        // 14. Return index.
        Ok(index)
    }

    /// Concrete method [`Evaluate ( )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-moduleevaluation
    pub(super) fn evaluate(&self, module_self: &Module, context: &mut Context) -> JsPromise {
        // 1. Assert: This call to Evaluate is not happening at the same time as another call to Evaluate within the surrounding agent.
        let (module, promise) = {
            match &*self.status.borrow() {
                ModuleStatus::Unlinked
                | ModuleStatus::Linking { .. }
                | ModuleStatus::PreLinked { .. }
                | ModuleStatus::Evaluating { .. } => {
                    unreachable!("2. Assert: module.[[Status]] is one of linked, evaluating-async, or evaluated.")
                }
                ModuleStatus::Linked { .. } => (module_self.clone(), None),
                // 3. If module.[[Status]] is either evaluating-async or evaluated, set module to module.[[CycleRoot]].
                ModuleStatus::EvaluatingAsync {
                    cycle_root,
                    top_level_capability,
                    ..
                }
                | ModuleStatus::Evaluated {
                    cycle_root,
                    top_level_capability,
                    ..
                } => (
                    cycle_root.clone(),
                    top_level_capability.as_ref().map(|cap| {
                        JsPromise::from_object(cap.promise().clone())
                            .expect("promise created from the %Promise% intrinsic is always native")
                    }),
                ),
            }
        };

        // 4. If module.[[TopLevelCapability]] is not empty, then
        if let Some(promise) = promise {
            // a. Return module.[[TopLevelCapability]].[[Promise]].
            return promise;
        }

        // 5. Let stack be a new empty List.
        let mut stack = Vec::new();

        // 6. Let capability be ! NewPromiseCapability(%Promise%).
        // 7. Set module.[[TopLevelCapability]] to capability.
        let capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("capability creation must always succeed when using the `%Promise%` intrinsic");

        // 8. Let result be Completion(InnerModuleEvaluation(module, stack, 0)).
        let ModuleKind::SourceText(module_src) = module.kind() else {
            unreachable!("module must be a source text module");
        };
        let result =
            module_src.inner_evaluate(&module, &mut stack, 0, Some(capability.clone()), context);

        match result {
            Ok(_) => {
                // 10. Else,
                //     a. Assert: module.[[Status]] is either evaluating-async or evaluated.
                assert!(match &*module_src.status.borrow() {
                    ModuleStatus::EvaluatingAsync { .. } => true,
                    // b. Assert: module.[[EvaluationError]] is empty.
                    ModuleStatus::Evaluated { error, .. } if error.is_none() => true,
                    _ => false,
                });

                //     c. If module.[[AsyncEvaluation]] is false, then
                if matches!(&*module_src.status.borrow(), ModuleStatus::Evaluated { .. }) {
                    //    i. Assert: module.[[Status]] is evaluated.
                    //    ii. Perform ! Call(capability.[[Resolve]], undefined, « undefined »).
                    capability
                        .resolve()
                        .call(&JsValue::undefined(), &[], context)
                        .expect("cannot fail for the default resolve function");
                }

                //     d. Assert: stack is empty.
                assert!(stack.is_empty());
            }
            // 9. If result is an abrupt completion, then
            Err(err) => {
                // a. For each Cyclic Module Record m of stack, do
                for m in stack.iter().filter_map(|cmr| cmr.kind().as_source_text()) {
                    m.status.borrow_mut().transition(|current| match current {
                        // i. Assert: m.[[Status]] is evaluating.
                        ModuleStatus::Evaluating {
                            environment,
                            top_level_capability,
                            cycle_root,
                            ..
                        } | ModuleStatus::EvaluatingAsync {
                            environment,
                            top_level_capability,
                            cycle_root,
                            ..
                        } => {
                            // ii. Set m.[[Status]] to evaluated.
                            // iii. Set m.[[EvaluationError]] to result.
                            ModuleStatus::Evaluated {
                                environment,
                                top_level_capability,
                                cycle_root,
                                error: Some(err.clone()),
                            }
                        },
                        _ => panic!(
                            "can only transition to `Evaluated` from the `Evaluating` or `EvaluatingAsync states"
                        ),
                    });
                }
                // b. Assert: module.[[Status]] is evaluated.
                // c. Assert: module.[[EvaluationError]] is result.
                assert!(
                    matches!(&*module_src.status.borrow(), ModuleStatus::Evaluated { error, .. } if error.is_some())
                );

                // d. Perform ! Call(capability.[[Reject]], undefined, « result.[[Value]] »).
                capability
                    .reject()
                    .call(&JsValue::undefined(), &[err.to_opaque(context)], context)
                    .expect("cannot fail for the default reject function");
            }
        }

        // 11. Return capability.[[Promise]].
        JsPromise::from_object(capability.promise().clone())
            .expect("promise created from the %Promise% intrinsic is always native")
    }

    /// Abstract operation [`InnerModuleEvaluation ( module, stack, index )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-innermoduleevaluation
    pub(super) fn inner_evaluate(
        &self,
        module_self: &Module,
        stack: &mut Vec<Module>,
        mut index: usize,
        capability: Option<PromiseCapability>,
        context: &mut Context,
    ) -> JsResult<usize> {
        /// Gets the next evaluation index of an async module.
        ///
        /// Returns an error if there's no more available indices.
        fn get_async_eval_index() -> JsResult<usize> {
            thread_local! {
                static ASYNC_EVAL_QUEUE_INDEX: Cell<usize> = const { Cell::new(0) };
            }

            ASYNC_EVAL_QUEUE_INDEX
                .with(|idx| {
                    let next = idx.get().checked_add(1)?;
                    Some(idx.replace(next))
                })
                .ok_or_else(|| {
                    JsNativeError::range()
                        .with_message("exceeded the maximum number of async modules")
                        .into()
                })
        }

        // 2. If module.[[Status]] is either evaluating-async or evaluated, then
        match &*self.status.borrow() {
            // 3. If module.[[Status]] is evaluating, return index.
            ModuleStatus::Evaluating { .. } | ModuleStatus::EvaluatingAsync { .. } => {
                return Ok(index)
            }
            //     a. If module.[[EvaluationError]] is empty, return index.
            //     b. Otherwise, return ? module.[[EvaluationError]].
            ModuleStatus::Evaluated { error, .. } => return error.clone().map_or(Ok(index), Err),
            ModuleStatus::Linked { .. } => {
                // 4. Assert: module.[[Status]] is linked.
                // evaluate a linked module
            }
            _ => unreachable!(
                "2. Assert: module.[[Status]] is one of linked, evaluating-async, or evaluated."
            ),
        }

        // 5. Set module.[[Status]] to evaluating.
        // 6. Set module.[[DFSIndex]] to index.
        // 7. Set module.[[DFSAncestorIndex]] to index.
        // 8. Set module.[[PendingAsyncDependencies]] to 0.
        self.status.borrow_mut().transition(|status| match status {
            ModuleStatus::Linked {
                environment,
                context,
                ..
            } => ModuleStatus::Evaluating {
                environment,
                context,
                top_level_capability: capability,
                cycle_root: module_self.clone(),
                info: DfsInfo {
                    dfs_index: index,
                    dfs_ancestor_index: index,
                },
                async_eval_index: None,
            },
            _ => unreachable!("already asserted that this state is `Linked`. "),
        });

        // 9. Set index to index + 1.
        index += 1;

        let mut pending_async_dependencies = 0;
        // 10. Append module to stack.
        stack.push(module_self.clone());

        // 11. For each String required of module.[[RequestedModules]], do
        for required in &self.code.requested_modules {
            // a. Let requiredModule be GetImportedModule(module, required).
            let required_module = self.loaded_modules.borrow()[required].clone();
            // b. Set index to ? InnerModuleEvaluation(requiredModule, stack, index).
            index = required_module.inner_evaluate(stack, index, context)?;

            // c. If requiredModule is a Cyclic Module Record, then
            if let ModuleKind::SourceText(required_module_src) = required_module.kind() {
                // i. Assert: requiredModule.[[Status]] is one of evaluating, evaluating-async, or evaluated.
                // ii. Assert: requiredModule.[[Status]] is evaluating if and only if stack contains requiredModule.
                debug_assert!(match &*required_module_src.status.borrow() {
                    ModuleStatus::EvaluatingAsync { .. } | ModuleStatus::Evaluated { .. } => true,
                    ModuleStatus::Evaluating { .. } if stack.contains(&required_module) => true,
                    _ => false,
                });

                let (required_module, async_eval, req_info) = match &*required_module_src.status.borrow() {
                    // iii. If requiredModule.[[Status]] is evaluating, then
                    ModuleStatus::Evaluating {
                        info,
                        async_eval_index,
                        ..
                    } => {
                        // 1. Set module.[[DFSAncestorIndex]] to min(module.[[DFSAncestorIndex]], requiredModule.[[DFSAncestorIndex]]).
                        (required_module.clone(), async_eval_index.is_some(), Some(*info))
                    }
                    // iv. Else,
                    ModuleStatus::EvaluatingAsync { cycle_root, .. }
                    | ModuleStatus::Evaluated { cycle_root, .. } => {
                        // 1. Set requiredModule to requiredModule.[[CycleRoot]].
                        let ModuleKind::SourceText(cycle_root_src) = cycle_root.kind() else {
                            unreachable!("cycle_root must be a source text module");
                        };

                        // 2. Assert: requiredModule.[[Status]] is either evaluating-async or evaluated.
                        match &*cycle_root_src.status.borrow() {
                            ModuleStatus::EvaluatingAsync { .. } => (cycle_root.clone(), true, None),
                            // 3. If requiredModule.[[EvaluationError]] is not empty, return ? requiredModule.[[EvaluationError]].
                            ModuleStatus::Evaluated { error: Some(error), .. } => return Err(error.clone()),
                            ModuleStatus::Evaluated { .. } => (cycle_root.clone(), false, None),
                            _ => unreachable!("2. Assert: requiredModule.[[Status]] is either evaluating-async or evaluated."),
                        }
                    }
                    _ => unreachable!("i. Assert: requiredModule.[[Status]] is one of evaluating, evaluating-async, or evaluated."),
                };

                let ModuleKind::SourceText(required_module) = required_module.kind() else {
                    unreachable!("required_module must be a source text module");
                };

                if let Some(req_info) = req_info {
                    let mut status = self.status.borrow_mut();
                    let info = status
                        .dfs_info_mut()
                        .expect("self should still be in the evaluating state");
                    info.dfs_ancestor_index =
                        usize::min(info.dfs_ancestor_index, req_info.dfs_ancestor_index);
                }

                // v. If requiredModule.[[AsyncEvaluation]] is true, then
                if async_eval {
                    // 1. Set module.[[PendingAsyncDependencies]] to module.[[PendingAsyncDependencies]] + 1.
                    pending_async_dependencies += 1;
                    // 2. Append module to requiredModule.[[AsyncParentModules]].
                    required_module
                        .async_parent_modules
                        .borrow_mut()
                        .push(module_self.clone());
                }
            }
        }

        // 12. If module.[[PendingAsyncDependencies]] > 0 or module.[[HasTLA]] is true, then
        if pending_async_dependencies > 0 || self.code.has_tla {
            // a. Assert: module.[[AsyncEvaluation]] is false and was never previously set to true.
            {
                let ModuleStatus::Evaluating {
                    async_eval_index, ..
                } = &mut *self.status.borrow_mut()
                else {
                    unreachable!("self should still be in the evaluating state")
                };

                // b. Set module.[[AsyncEvaluation]] to true.
                // c. NOTE: The order in which module records have their [[AsyncEvaluation]] fields transition to true is significant. (See 16.2.1.5.3.4.)
                *async_eval_index = Some(get_async_eval_index()?);
            }

            //     d. If module.[[PendingAsyncDependencies]] = 0, perform ExecuteAsyncModule(module).
            if pending_async_dependencies == 0 {
                self.execute_async(module_self, context);
            }
        } else {
            // 13. Else,
            //    a. Perform ? module.ExecuteModule().
            self.execute(module_self, None, context)?;
        }

        let dfs_info = self.status.borrow().dfs_info().copied().expect(
            "haven't transitioned from the `Evaluating` state, so it should have its dfs info",
        );

        // 14. Assert: module occurs exactly once in stack.
        debug_assert_eq!(stack.iter().filter(|m| *m == module_self).count(), 1);
        // 15. Assert: module.[[DFSAncestorIndex]] ≤ module.[[DFSIndex]].
        assert!(dfs_info.dfs_ancestor_index <= dfs_info.dfs_index);

        // 16. If module.[[DFSAncestorIndex]] = module.[[DFSIndex]], then
        if dfs_info.dfs_ancestor_index == dfs_info.dfs_index {
            // a. Let done be false.
            // b. Repeat, while done is false,
            loop {
                // i. Let requiredModule be the last element of stack.
                // ii. Remove the last element of stack.
                let required_module = stack
                    .pop()
                    .expect("should at least have `self` in the stack");
                let is_self = module_self == &required_module;

                let ModuleKind::SourceText(required_module_src) = required_module.kind() else {
                    unreachable!("iii. Assert: requiredModule is a Cyclic Module Record.");
                };
                required_module_src.status.borrow_mut().transition(|current| match current {
                ModuleStatus::Evaluating {
                            environment,
                            top_level_capability,
                            cycle_root,
                            async_eval_index,
                            context,
                            ..
                        } => if let Some(async_eval_index) = async_eval_index {
                            // v. Otherwise, set requiredModule.[[Status]] to evaluating-async.
                            ModuleStatus::EvaluatingAsync {
                                environment,
                                top_level_capability,
                                // vii. Set requiredModule.[[CycleRoot]] to module.
                                cycle_root: if is_self {
                                    cycle_root
                                } else {
                                    module_self.clone()
                                },
                                async_eval_index,
                                pending_async_dependencies,
                                context
                            }
                        } else {
                            // iv. If requiredModule.[[AsyncEvaluation]] is false, set requiredModule.[[Status]] to evaluated.
                            ModuleStatus::Evaluated {
                                environment,
                                top_level_capability,
                                cycle_root: if is_self {
                                    cycle_root
                                } else {
                                    module_self.clone()
                                },
                                error: None,
                            }
                        }
                        _ => unreachable!(
                            "should only transition to `Evaluated` or `EvaluatingAsync` from the `Evaluating` state"
                        )
                    }
                );

                // vi. If requiredModule and module are the same Module Record, set done to true.
                if is_self {
                    break;
                }
            }
        }

        // 17. Return index.
        Ok(index)
    }

    /// Abstract operation [`ExecuteAsyncModule ( module )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-execute-async-module
    fn execute_async(&self, module_self: &Module, context: &mut Context) {
        // 1. Assert: module.[[Status]] is either evaluating or evaluating-async.
        debug_assert!(matches!(
            &*self.status.borrow(),
            ModuleStatus::Evaluating { .. } | ModuleStatus::EvaluatingAsync { .. }
        ));
        // 2. Assert: module.[[HasTLA]] is true.
        debug_assert!(self.code.has_tla);

        // 3. Let capability be ! NewPromiseCapability(%Promise%).
        let capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("cannot fail for the %Promise% intrinsic");

        // 4. Let fulfilledClosure be a new Abstract Closure with no parameters that captures module and performs the following steps when called:
        // 5. Let onFulfilled be CreateBuiltinFunction(fulfilledClosure, 0, "", « »).
        let on_fulfilled = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_copy_closure_with_captures(
                |_, _, module, context| {
                    //     a. Perform AsyncModuleExecutionFulfilled(module).
                    async_module_execution_fulfilled(module, context);
                    //     b. Return undefined.
                    Ok(JsValue::undefined())
                },
                module_self.clone(),
            ),
        )
        .build();

        // 6. Let rejectedClosure be a new Abstract Closure with parameters (error) that captures module and performs the following steps when called:
        // 7. Let onRejected be CreateBuiltinFunction(rejectedClosure, 0, "", « »).
        let on_rejected = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_copy_closure_with_captures(
                |_, args, module, context| {
                    let error = JsError::from_opaque(args.get_or_undefined(0).clone());
                    // a. Perform AsyncModuleExecutionRejected(module, error).
                    async_module_execution_rejected(module, &error, context);
                    // b. Return undefined.
                    Ok(JsValue::undefined())
                },
                module_self.clone(),
            ),
        )
        .build();

        // 8. Perform PerformPromiseThen(capability.[[Promise]], onFulfilled, onRejected).
        Promise::perform_promise_then(
            capability.promise(),
            Some(on_fulfilled),
            Some(on_rejected),
            None,
            context,
        );

        // 9. Perform ! module.ExecuteModule(capability).
        // 10. Return unused.
        self.execute(module_self, Some(&capability), context)
            .expect("async modules cannot directly throw");
    }

    /// Abstract operation [`GatherAvailableAncestors ( module, execList )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-gather-available-ancestors
    #[allow(clippy::mutable_key_type)]
    fn gather_available_ancestors(&self, exec_list: &mut FxHashSet<Module>) {
        // 1. For each Cyclic Module Record m of module.[[AsyncParentModules]], do
        for m in &*self.async_parent_modules.borrow() {
            let ModuleKind::SourceText(m_src) = m.kind() else {
                continue;
            };

            if exec_list.contains(m) {
                continue;
            }

            {
                let m_status = m_src.status.borrow();
                let Some(cycle_root) = m_status
                    .cycle_root()
                    .and_then(|root| root.kind().as_source_text())
                else {
                    continue;
                };

                if cycle_root.status.borrow().evaluation_error().is_some() {
                    continue;
                }
            }

            // a. If execList does not contain m and m.[[CycleRoot]].[[EvaluationError]] is empty, then

            let (deps, has_tla) = {
                // i. Assert: m.[[Status]] is evaluating-async.
                // ii. Assert: m.[[EvaluationError]] is empty.
                // iii. Assert: m.[[AsyncEvaluation]] is true.
                let ModuleStatus::EvaluatingAsync {
                    pending_async_dependencies,
                    ..
                } = &mut *m_src.status.borrow_mut()
                else {
                    unreachable!("i. Assert: m.[[Status]] is evaluating-async.");
                };
                // iv. Assert: m.[[PendingAsyncDependencies]] > 0.
                assert!(*pending_async_dependencies > 0);

                // v. Set m.[[PendingAsyncDependencies]] to m.[[PendingAsyncDependencies]] - 1.
                *pending_async_dependencies -= 1;
                (*pending_async_dependencies, m_src.code.has_tla)
            };

            // vi. If m.[[PendingAsyncDependencies]] = 0, then
            if deps == 0 {
                // 1. Append m to execList.
                exec_list.insert(m.clone());
                // 2. If m.[[HasTLA]] is false, perform GatherAvailableAncestors(m, execList).
                if !has_tla {
                    m_src.gather_available_ancestors(exec_list);
                }
            }
        }
        // 2. Return unused.
    }

    /// Abstract operation [`InitializeEnvironment ( )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-source-text-module-record-initialize-environment
    fn initialize_environment(&self, module_self: &Module, context: &mut Context) -> JsResult<()> {
        #[derive(Debug)]
        enum ImportBinding {
            Namespace {
                locator: BindingLocator,
                module: Module,
            },
            Single {
                locator: BindingLocator,
                export_locator: ResolvedBinding,
            },
        }

        {
            // 1. For each ExportEntry Record e of module.[[IndirectExportEntries]], do
            for e in &self.code.indirect_export_entries {
                // a. Let resolution be module.ResolveExport(e.[[ExportName]]).
                module_self
                    .resolve_export(
                        e.export_name().to_js_string(context.interner()),
                        &mut HashSet::default(),
                        context.interner(),
                    )
                    // b. If resolution is either null or ambiguous, throw a SyntaxError exception.
                    .map_err(|err| match err {
                        ResolveExportError::NotFound => {
                            JsNativeError::syntax().with_message(format!(
                                "could not find export `{}`",
                                context.interner().resolve_expect(e.export_name())
                            ))
                        }
                        ResolveExportError::Ambiguous => {
                            JsNativeError::syntax().with_message(format!(
                                "could not resolve ambiguous export `{}`",
                                context.interner().resolve_expect(e.export_name())
                            ))
                        }
                    })?;
                // c. Assert: resolution is a ResolvedBinding Record.
            }
        }

        // 2. Assert: All named exports from module are resolvable.
        // 3. Let realm be module.[[Realm]].
        // 4. Assert: realm is not undefined.
        let realm = module_self.realm().clone();

        // 5. Let env be NewModuleEnvironment(realm.[[GlobalEnv]]).
        // 6. Set module.[[Environment]] to env.
        let global_env = realm.environment().clone();
        let global_compile_env = global_env.compile_env();
        let env = Rc::new(CompileTimeEnvironment::new(global_compile_env, true));

        let mut compiler = ByteCompiler::new(
            js_string!("<main>"),
            true,
            false,
            env.clone(),
            env.clone(),
            context.interner_mut(),
        );

        compiler.code_block_flags |= CodeBlockFlags::IS_ASYNC;
        compiler.async_handler = Some(compiler.push_handler());

        let mut imports = Vec::new();

        let (codeblock, functions) = {
            // 7. For each ImportEntry Record in of module.[[ImportEntries]], do
            for entry in &self.code.import_entries {
                // a. Let importedModule be GetImportedModule(module, in.[[ModuleRequest]]).
                let module_request = entry.module_request().to_js_string(compiler.interner());
                let imported_module = self.loaded_modules.borrow()[&module_request].clone();

                if let ImportName::Name(name) = entry.import_name() {
                    let name = name.to_js_string(compiler.interner());
                    // c. Else,
                    //    i. Let resolution be importedModule.ResolveExport(in.[[ImportName]]).
                    let resolution = imported_module
                        .resolve_export(name.clone(), &mut HashSet::default(), compiler.interner())
                        // ii. If resolution is either null or ambiguous, throw a SyntaxError exception.
                        .map_err(|err| match err {
                            ResolveExportError::NotFound => JsNativeError::syntax().with_message(
                                format!("could not find export `{}`", name.to_std_string_escaped()),
                            ),
                            ResolveExportError::Ambiguous => {
                                JsNativeError::syntax().with_message(format!(
                                    "could not resolve ambiguous export `{}`",
                                    name.to_std_string_escaped()
                                ))
                            }
                        })?;

                    // 2. Perform ! env.CreateImmutableBinding(in.[[LocalName]], true).
                    // 3. Perform ! env.InitializeBinding(in.[[LocalName]], namespace).
                    let local_name = entry.local_name().to_js_string(compiler.interner());
                    let locator = env.create_immutable_binding(local_name, true);

                    if let BindingName::Name(_) = resolution.binding_name {
                        // 1. Perform env.CreateImportBinding(in.[[LocalName]], resolution.[[Module]],
                        //    resolution.[[BindingName]]).
                        //    deferred to initialization below
                        imports.push(ImportBinding::Single {
                            locator,
                            export_locator: resolution,
                        });
                    } else {
                        // 1. Let namespace be GetModuleNamespace(resolution.[[Module]]).
                        // deferred to initialization below
                        imports.push(ImportBinding::Namespace {
                            locator,
                            module: resolution.module,
                        });
                    }
                } else {
                    // b. If in.[[ImportName]] is namespace-object, then
                    //    ii. Perform ! env.CreateImmutableBinding(in.[[LocalName]], true).
                    //    iii. Perform ! env.InitializeBinding(in.[[LocalName]], namespace).
                    let locator = env.create_immutable_binding(
                        entry.local_name().to_js_string(compiler.interner()),
                        true,
                    );

                    //    i. Let namespace be GetModuleNamespace(importedModule).
                    //       deferred to initialization below
                    imports.push(ImportBinding::Namespace {
                        locator,
                        module: imported_module.clone(),
                    });
                }
            }

            // 18. Let code be module.[[ECMAScriptCode]].
            // 19. Let varDeclarations be the VarScopedDeclarations of code.
            let var_declarations = var_scoped_declarations(&self.code.source);
            // 20. Let declaredVarNames be a new empty List.
            let mut declared_var_names = Vec::new();
            // 21. For each element d of varDeclarations, do
            for var in var_declarations {
                // a. For each element dn of the BoundNames of d, do
                for name in var.bound_names() {
                    let name = name.to_js_string(compiler.interner());

                    // i. If declaredVarNames does not contain dn, then
                    if !declared_var_names.contains(&name) {
                        // 1. Perform ! env.CreateMutableBinding(dn, false).
                        // 2. Perform ! env.InitializeBinding(dn, undefined).
                        let binding = env.create_mutable_binding(name.clone(), false);
                        let index = compiler.get_or_insert_binding(binding);
                        compiler.emit_opcode(Opcode::PushUndefined);
                        compiler.emit_with_varying_operand(Opcode::DefInitVar, index);

                        // 3. Append dn to declaredVarNames.
                        declared_var_names.push(name);
                    }
                }
            }

            // 22. Let lexDeclarations be the LexicallyScopedDeclarations of code.
            // 23. Let privateEnv be null.
            let lex_declarations = lexically_scoped_declarations(&self.code.source);
            let mut functions = Vec::new();
            // 24. For each element d of lexDeclarations, do
            for declaration in lex_declarations {
                // ii. Else,
                // a. For each element dn of the BoundNames of d, do
                // 1. Perform ! env.CreateMutableBinding(dn, false).
                //
                // iii. If d is either a FunctionDeclaration, a GeneratorDeclaration, an
                //      AsyncFunctionDeclaration, or an AsyncGeneratorDeclaration, then
                // 1. Let fo be InstantiateFunctionObject of d with arguments env and privateEnv.
                // 2. Perform ! env.InitializeBinding(dn, fo).
                //
                // deferred to below.
                let (spec, locator): (FunctionSpec<'_>, _) = match declaration {
                    LexicallyScopedDeclaration::Function(f) => {
                        let name = bound_names(f)[0].to_js_string(compiler.interner());
                        let locator = env.create_mutable_binding(name, false);

                        (f.into(), locator)
                    }
                    LexicallyScopedDeclaration::Generator(g) => {
                        let name = bound_names(g)[0].to_js_string(compiler.interner());
                        let locator = env.create_mutable_binding(name, false);

                        (g.into(), locator)
                    }
                    LexicallyScopedDeclaration::AsyncFunction(af) => {
                        let name = bound_names(af)[0].to_js_string(compiler.interner());
                        let locator = env.create_mutable_binding(name, false);

                        (af.into(), locator)
                    }
                    LexicallyScopedDeclaration::AsyncGenerator(ag) => {
                        let name = bound_names(ag)[0].to_js_string(compiler.interner());
                        let locator = env.create_mutable_binding(name, false);

                        (ag.into(), locator)
                    }
                    LexicallyScopedDeclaration::Class(class) => {
                        for name in bound_names(class) {
                            let name = name.to_js_string(compiler.interner());
                            env.create_mutable_binding(name, false);
                        }
                        continue;
                    }
                    // i. If IsConstantDeclaration of d is true, then
                    LexicallyScopedDeclaration::LexicalDeclaration(LexicalDeclaration::Const(
                        c,
                    )) => {
                        // a. For each element dn of the BoundNames of d, do
                        for name in bound_names(c) {
                            let name = name.to_js_string(compiler.interner());
                            // 1. Perform ! env.CreateImmutableBinding(dn, true).
                            env.create_immutable_binding(name, true);
                        }
                        continue;
                    }
                    LexicallyScopedDeclaration::LexicalDeclaration(LexicalDeclaration::Let(l)) => {
                        for name in bound_names(l) {
                            let name = name.to_js_string(compiler.interner());
                            env.create_mutable_binding(name, false);
                        }
                        continue;
                    }
                    LexicallyScopedDeclaration::AssignmentExpression(expr) => {
                        for name in bound_names(expr) {
                            let name = name.to_js_string(compiler.interner());
                            env.create_mutable_binding(name, false);
                        }
                        continue;
                    }
                };

                functions.push((spec, locator));
            }

            // Should compile after initializing bindings first to ensure inner calls
            // are correctly resolved to the outer functions instead of as global bindings.
            let functions = functions
                .into_iter()
                .map(|(spec, locator)| (compiler.function(spec), locator))
                .collect::<Vec<_>>();

            compiler.compile_module_item_list(self.code.source.items());

            (Gc::new(compiler.finish()), functions)
        };

        // 8. Let moduleContext be a new ECMAScript code execution context.
        let mut envs = EnvironmentStack::new(global_env);
        envs.push_module(env);

        // 9. Set the Function of moduleContext to null.
        // 10. Assert: module.[[Realm]] is not undefined.
        // 11. Set the Realm of moduleContext to module.[[Realm]].
        // 12. Set the ScriptOrModule of moduleContext to module.
        // 13. Set the VariableEnvironment of moduleContext to module.[[Environment]].
        // 14. Set the LexicalEnvironment of moduleContext to module.[[Environment]].
        // 15. Set the PrivateEnvironment of moduleContext to null.
        let call_frame = CallFrame::new(
            codeblock.clone(),
            Some(ActiveRunnable::Module(module_self.clone())),
            envs,
            realm.clone(),
        );
        context
            .vm
            .push_frame_with_stack(call_frame, JsValue::undefined(), JsValue::null());

        // 17. Push moduleContext onto the execution context stack; moduleContext is now the running execution context.

        // deferred initialization of import bindings
        for import in imports {
            match import {
                ImportBinding::Namespace { locator, module } => {
                    // i. Let namespace be GetModuleNamespace(importedModule).
                    let namespace = module.namespace(context);
                    context.vm.environments.put_lexical_value(
                        locator.environment_index(),
                        locator.binding_index(),
                        namespace.into(),
                    );
                }
                ImportBinding::Single {
                    locator,
                    export_locator,
                } => match export_locator.binding_name() {
                    BindingName::Name(name) => context
                        .vm
                        .environments
                        .current()
                        .declarative_expect()
                        .kind()
                        .as_module()
                        .expect("last environment should be the module env")
                        .set_indirect(locator.binding_index(), export_locator.module, name),
                    BindingName::Namespace => {
                        let namespace = export_locator.module.namespace(context);
                        context.vm.environments.put_lexical_value(
                            locator.environment_index(),
                            locator.binding_index(),
                            namespace.into(),
                        );
                    }
                },
            }
        }

        // deferred initialization of function exports
        for (index, locator) in functions {
            let code = codeblock.constant_function(index as usize);

            let function = create_function_object_fast(code, context);

            context.vm.environments.put_lexical_value(
                locator.environment_index(),
                locator.binding_index(),
                function.into(),
            );
        }

        // 25. Remove moduleContext from the execution context stack.
        let frame = context
            .vm
            .pop_frame()
            .expect("There should be a call frame");

        let env = frame
            .environments
            .current()
            .as_declarative()
            .cloned()
            .expect("frame must have a declarative environment");

        // 16. Set module.[[Context]] to moduleContext.
        self.status.borrow_mut().transition(|state| match state {
            ModuleStatus::Linking { info } => ModuleStatus::PreLinked {
                environment: env,
                info,
                context: SourceTextContext {
                    codeblock,
                    environments: frame.environments.clone(),
                    realm,
                },
            },
            _ => unreachable!(
                "should only transition to the `PreLinked` state from the `Linking` state"
            ),
        });

        // 26. Return unused.
        Ok(())
    }

    /// Abstract operation [`ExecuteModule ( [ capability ] )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-source-text-module-record-execute-module
    fn execute(
        &self,
        module_self: &Module,
        capability: Option<&PromiseCapability>,
        context: &mut Context,
    ) -> JsResult<()> {
        // 1. Let moduleContext be a new ECMAScript code execution context.
        let SourceTextContext {
            codeblock,
            environments,
            realm,
        } = match &*self.status.borrow() {
            ModuleStatus::Evaluating { context, .. }
            | ModuleStatus::EvaluatingAsync { context, .. } => context.clone(),
            _ => unreachable!("`execute` should only be called for evaluating modules."),
        };

        // 2. Set the Function of moduleContext to null.
        // 3. Set the Realm of moduleContext to module.[[Realm]].
        // 4. Set the ScriptOrModule of moduleContext to module.
        // 5. Assert: module has been linked and declarations in its module environment have been instantiated.
        // 6. Set the VariableEnvironment of moduleContext to module.[[Environment]].
        // 7. Set the LexicalEnvironment of moduleContext to module.[[Environment]].
        let env_fp = environments.len() as u32;
        let callframe = CallFrame::new(
            codeblock,
            Some(ActiveRunnable::Module(module_self.clone())),
            environments,
            realm,
        )
        .with_env_fp(env_fp)
        .with_flags(CallFrameFlags::EXIT_EARLY);

        // 8. Suspend the running execution context.
        context
            .vm
            .push_frame_with_stack(callframe, JsValue::undefined(), JsValue::null());

        context
            .vm
            .frames
            .last()
            .expect("there should be a frame")
            .set_promise_capability(&mut context.vm.stack, capability);

        // 9. If module.[[HasTLA]] is false, then
        //    a. Assert: capability is not present.
        //    b. Push moduleContext onto the execution context stack; moduleContext is now the running execution context.
        //    c. Let result be Completion(Evaluation of module.[[ECMAScriptCode]]).
        //    d. Suspend moduleContext and remove it from the execution context stack.
        //    e. Resume the context that is now on the top of the execution context stack as the running execution context.
        // 10. Else,
        //    a. Assert: capability is a PromiseCapability Record.
        //    b. Perform AsyncBlockStart(capability, module.[[ECMAScriptCode]], moduleContext).
        let result = context.run();

        context.vm.pop_frame();

        //     f. If result is an abrupt completion, then
        if let CompletionRecord::Throw(err) = result {
            //    i. Return ? result.
            Err(err)
        } else {
            // 11. Return unused.
            Ok(())
        }
    }

    /// Gets the loaded modules of this module.
    pub(crate) fn loaded_modules(&self) -> &GcRefCell<FxHashMap<JsString, Module>> {
        &self.loaded_modules
    }

    /// Gets the import meta object of this module, or initializes
    /// it using the provided callback.
    pub(crate) fn import_meta(&self) -> &GcRefCell<Option<JsObject>> {
        &self.import_meta
    }

    /// Gets the declarative environment of this module.
    pub(crate) fn environment(&self) -> Option<Gc<DeclarativeEnvironment>> {
        self.status.borrow().environment()
    }
}

/// Abstract operation [`AsyncModuleExecutionFulfilled ( module )`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-async-module-execution-fulfilled
#[allow(clippy::mutable_key_type)]
fn async_module_execution_fulfilled(module: &Module, context: &mut Context) {
    let ModuleKind::SourceText(module_src) = module.kind() else {
        unreachable!("async executed module must be a source text module");
    };

    // 1. If module.[[Status]] is evaluated, then
    if let ModuleStatus::Evaluated { error, .. } = &*module_src.status.borrow() {
        //     a. Assert: module.[[EvaluationError]] is not empty.
        assert!(error.is_some());
        //     b. Return unused.
        return;
    }

    // 2. Assert: module.[[Status]] is evaluating-async.
    // 3. Assert: module.[[AsyncEvaluation]] is true.
    // 4. Assert: module.[[EvaluationError]] is empty.
    // 5. Set module.[[AsyncEvaluation]] to false.
    // 6. Set module.[[Status]] to evaluated.
    module_src
        .status
        .borrow_mut()
        .transition(|status| match status {
            ModuleStatus::EvaluatingAsync {
                environment,
                top_level_capability,
                cycle_root,
                ..
            } => ModuleStatus::Evaluated {
                environment,
                top_level_capability,
                cycle_root,
                error: None,
            },
            _ => unreachable!(),
        });

    // 7. If module.[[TopLevelCapability]] is not empty, then
    if let Some(cap) = module_src.status.borrow().top_level_capability() {
        // a. Assert: module.[[CycleRoot]] is module.
        debug_assert_eq!(module_src.status.borrow().cycle_root(), Some(module));

        // b. Perform ! Call(module.[[TopLevelCapability]].[[Resolve]], undefined, « undefined »).
        cap.resolve()
            .call(&JsValue::undefined(), &[], context)
            .expect("default `resolve` function cannot fail");
    }

    // 8. Let execList be a new empty List.
    let mut ancestors = FxHashSet::default();

    // 9. Perform GatherAvailableAncestors(module, execList).
    module_src.gather_available_ancestors(&mut ancestors);

    // 11. Assert: All elements of sortedExecList have their [[AsyncEvaluation]] field set to true, [[PendingAsyncDependencies]] field set to 0, and [[EvaluationError]] field set to empty.
    let mut ancestors = ancestors.into_iter().collect::<Vec<_>>();

    // 10. Let sortedExecList be a List whose elements are the elements of execList, in the order in which they had their [[AsyncEvaluation]] fields set to true in InnerModuleEvaluation.
    ancestors.sort_by_cached_key(|m| {
        let ModuleKind::SourceText(m_src) = m.kind() else {
            unreachable!("ancestors must only be source text modules");
        };

        let ModuleStatus::EvaluatingAsync {
            async_eval_index, ..
        } = &*m_src.status.borrow()
        else {
            unreachable!("GatherAvailableAncestors: i. Assert: m.[[Status]] is evaluating-async.");
        };

        *async_eval_index
    });

    // 12. For each Cyclic Module Record m of sortedExecList, do
    for m in ancestors {
        let ModuleKind::SourceText(m_src) = m.kind() else {
            continue;
        };

        // a. If m.[[Status]] is evaluated, then
        if let ModuleStatus::Evaluated { error, .. } = &*m_src.status.borrow() {
            // i. Assert: m.[[EvaluationError]] is not empty.
            assert!(error.is_some());
            continue;
        }

        // b. Else if m.[[HasTLA]] is true, then
        let has_tla = m_src.code.has_tla;
        if has_tla {
            // i. Perform ExecuteAsyncModule(m).
            m_src.execute_async(&m, context);
        } else {
            // c. Else,
            //    i. Let result be m.ExecuteModule().
            let result = m_src.execute(&m, None, context);

            //    ii. If result is an abrupt completion, then
            if let Err(e) = result {
                //    1. Perform AsyncModuleExecutionRejected(m, result.[[Value]]).
                async_module_execution_rejected(module, &e, context);
            } else {
                // iii. Else,
                //    1. Set m.[[Status]] to evaluated.
                m_src.status.borrow_mut().transition(|status| match status {
                    ModuleStatus::EvaluatingAsync {
                        environment,
                        top_level_capability,
                        cycle_root,
                        ..
                    } => ModuleStatus::Evaluated {
                        environment,
                        top_level_capability,
                        cycle_root,
                        error: None,
                    },
                    _ => unreachable!(),
                });

                let status = m_src.status.borrow();
                // 2. If m.[[TopLevelCapability]] is not empty, then
                if let Some(cap) = status.top_level_capability() {
                    // a. Assert: m.[[CycleRoot]] is m.
                    assert_eq!(status.cycle_root(), Some(&m));

                    // b. Perform ! Call(m.[[TopLevelCapability]].[[Resolve]], undefined, « undefined »).
                    cap.resolve()
                        .call(&JsValue::undefined(), &[], context)
                        .expect("default `resolve` function cannot fail");
                }
            }
        }
    }
    // 13. Return unused.
}

/// Abstract operation [`AsyncModuleExecutionRejected ( module, error )`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-async-module-execution-rejected
fn async_module_execution_rejected(module: &Module, error: &JsError, context: &mut Context) {
    let ModuleKind::SourceText(module_src) = module.kind() else {
        unreachable!("async executed module must be a source text module");
    };
    // 1. If module.[[Status]] is evaluated, then
    if let ModuleStatus::Evaluated { error, .. } = &*module_src.status.borrow() {
        //     a. Assert: module.[[EvaluationError]] is not empty.
        assert!(error.is_some());
        //     b. Return unused.
        return;
    }

    // 2. Assert: module.[[Status]] is evaluating-async.
    // 3. Assert: module.[[AsyncEvaluation]] is true.
    // 4. Assert: module.[[EvaluationError]] is empty.
    // 5. Set module.[[EvaluationError]] to ThrowCompletion(error).
    // 6. Set module.[[Status]] to evaluated.
    module_src
        .status
        .borrow_mut()
        .transition(|status| match status {
            ModuleStatus::EvaluatingAsync {
                environment,
                top_level_capability,
                cycle_root,
                ..
            } => ModuleStatus::Evaluated {
                environment,
                top_level_capability,
                cycle_root,
                error: Some(error.clone()),
            },
            _ => unreachable!(),
        });

    // 7. For each Cyclic Module Record m of module.[[AsyncParentModules]], do
    for m in &*module_src.async_parent_modules.borrow() {
        // a. Perform AsyncModuleExecutionRejected(m, error).
        async_module_execution_rejected(m, error, context);
    }

    let status = module_src.status.borrow();
    // 8. If module.[[TopLevelCapability]] is not empty, then
    if let Some(cap) = status.top_level_capability() {
        // a. Assert: module.[[CycleRoot]] is module.
        assert_eq!(status.cycle_root(), Some(module));

        // b. Perform ! Call(module.[[TopLevelCapability]].[[Reject]], undefined, « error »).
        cap.reject()
            .call(&JsValue::undefined(), &[error.to_opaque(context)], context)
            .expect("default `reject` function cannot fail");
    }
    // 9. Return unused.
}
