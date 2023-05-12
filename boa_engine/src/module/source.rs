use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    hash::Hash,
    rc::Rc,
};

use boa_ast::{
    declaration::{
        ExportEntry, ImportEntry, ImportName, IndirectExportEntry, LexicalDeclaration,
        LocalExportEntry, ReExportImportName,
    },
    operations::{
        bound_names, contains, lexically_scoped_declarations, var_scoped_declarations,
        ContainsSymbol,
    },
    Declaration, ModuleItemList,
};
use boa_gc::{custom_trace, empty_trace, Finalize, Gc, GcRef, GcRefCell, GcRefMut, Trace};
use boa_interner::Sym;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    builtins::{promise::PromiseCapability, Promise},
    bytecompiler::{ByteCompiler, NodeKind},
    environments::{BindingLocator, CompileTimeEnvironment, EnvironmentStack},
    module::ModuleKind,
    object::{FunctionObjectBuilder, JsPromise, RecursionLimiter},
    realm::Realm,
    vm::{CallFrame, CodeBlock, CompletionRecord, Opcode},
    Context, JsArgs, JsError, JsNativeError, JsObject, JsResult, JsString, JsValue, NativeFunction,
};

use super::{
    BindingName, GraphLoadingState, Module, Referrer, ResolveExportError, ResolvedBinding,
};

/// Information for the [**Depth-first search**] algorithm used in the
/// [`Module::link`] and [`Module::evaluate`] methods.
#[derive(Clone, Copy, Debug, Finalize)]
pub(super) struct DfsInfo {
    dfs_index: usize,
    dfs_ancestor_index: usize,
}

// SAFETY: `DfsInfo` only contains primitive types, making this safe.
unsafe impl Trace for DfsInfo {
    empty_trace!();
}

/// Current status of a [`SourceTextModule`].
///
/// Roughly corresponds to the `[[Status]]` field of [**Cyclic Module Records**][cyclic],
/// but with a state machine-like design for better correctness.
///
/// [cyclic]: https://tc39.es/ecma262/#table-cyclic-module-fields
#[derive(Debug, Finalize, Default)]
enum Status {
    #[default]
    Unlinked,
    Linking {
        info: DfsInfo,
    },
    PreLinked {
        context: SourceTextContext,
        info: DfsInfo,
    },
    Linked {
        context: SourceTextContext,
        info: DfsInfo,
    },
    Evaluating {
        context: SourceTextContext,
        top_level_capability: Option<PromiseCapability>,
        cycle_root: SourceTextModule,
        info: DfsInfo,
        async_eval_index: Option<usize>,
    },
    EvaluatingAsync {
        context: SourceTextContext,
        top_level_capability: Option<PromiseCapability>,
        cycle_root: SourceTextModule,
        async_eval_index: usize,
        pending_async_dependencies: usize,
    },
    Evaluated {
        top_level_capability: Option<PromiseCapability>,
        cycle_root: SourceTextModule,
        error: Option<JsError>,
    },
}

// SAFETY: This must be synced with `Status` to mark any new data added that needs to be traced.
// This implementation is necessary to be able to transition from one state to another by destructuring,
// which saves us some unnecessary clones.
unsafe impl Trace for Status {
    custom_trace!(this, {
        match this {
            Status::Unlinked | Status::Linking { .. } | Status::Linked { .. } => {}
            Status::PreLinked { context, .. } => mark(context),
            Status::Evaluating {
                top_level_capability,
                cycle_root,
                context,
                ..
            }
            | Status::EvaluatingAsync {
                top_level_capability,
                cycle_root,
                context,
                ..
            } => {
                mark(top_level_capability);
                mark(cycle_root);
                mark(context);
            }
            Status::Evaluated {
                top_level_capability,
                cycle_root,
                error,
            } => {
                mark(top_level_capability);
                mark(cycle_root);
                mark(error);
            }
        }
    });
}

impl Status {
    /// Gets the current index info of the module within the dependency graph, or `None` if the
    /// module is not in a state executing the dfs algorithm.
    const fn dfs_info(&self) -> Option<&DfsInfo> {
        match self {
            Status::Unlinked | Status::EvaluatingAsync { .. } | Status::Evaluated { .. } => None,
            Status::Linking { info }
            | Status::PreLinked { info, .. }
            | Status::Linked { info, .. }
            | Status::Evaluating { info, .. } => Some(info),
        }
    }

    /// Gets a mutable reference to the current index info of the module within the dependency graph,
    /// or `None` if the module is not in a state executing the dfs algorithm.
    fn dfs_info_mut(&mut self) -> Option<&mut DfsInfo> {
        match self {
            Status::Unlinked | Status::EvaluatingAsync { .. } | Status::Evaluated { .. } => None,
            Status::Linking { info }
            | Status::PreLinked { info, .. }
            | Status::Linked { info, .. }
            | Status::Evaluating { info, .. } => Some(info),
        }
    }

    /// If this module is the top module being evaluated and is in the evaluating state, gets its top
    /// level capability.
    const fn top_level_capability(&self) -> Option<&PromiseCapability> {
        match &self {
            Status::Unlinked
            | Status::Linking { .. }
            | Status::PreLinked { .. }
            | Status::Linked { .. } => None,
            Status::Evaluating {
                top_level_capability,
                ..
            }
            | Status::EvaluatingAsync {
                top_level_capability,
                ..
            }
            | Status::Evaluated {
                top_level_capability,
                ..
            } => top_level_capability.as_ref(),
        }
    }

    /// If this module is in the evaluated state, gets its `error` field.
    const fn evaluation_error(&self) -> Option<&JsError> {
        match &self {
            Status::Evaluated { error, .. } => error.as_ref(),
            _ => None,
        }
    }

    /// If this module is in the evaluating state, gets its cycle root.
    const fn cycle_root(&self) -> Option<&SourceTextModule> {
        match &self {
            Status::Evaluating { cycle_root, .. }
            | Status::EvaluatingAsync { cycle_root, .. }
            | Status::Evaluated { cycle_root, .. } => Some(cycle_root),
            _ => None,
        }
    }

    /// Transition from one state to another, taking the current state by value to move data
    /// between states.
    fn transition<F>(&mut self, f: F)
    where
        F: FnOnce(Status) -> Status,
    {
        *self = f(std::mem::take(self));
    }
}

/// The execution context of a [`SourceTextModule`].
///
/// Stores the required context data that needs to be in place before executing the
/// inner code of the module.
#[derive(Clone, Finalize)]
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

unsafe impl Trace for SourceTextContext {
    custom_trace!(this, {
        mark(&this.codeblock);
        mark(&this.environments);
        mark(&this.realm);
    });
}

/// ECMAScript's [**Source Text Module Records**][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-source-text-module-records
#[derive(Clone, Trace, Finalize)]
pub(crate) struct SourceTextModule(Gc<GcRefCell<Inner>>);

impl std::fmt::Debug for SourceTextModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let limiter = RecursionLimiter::new(&*self.0);

        if !limiter.visited && !limiter.live {
            f.debug_tuple("SourceTextModule").field(&self.0).finish()
        } else {
            f.debug_tuple("SourceTextModule").field(&"<cycle>").finish()
        }
    }
}

#[derive(Finalize)]
struct Inner {
    code: ModuleItemList,
    status: Status,
    requested_modules: FxHashSet<Sym>,
    loaded_modules: FxHashMap<Sym, Module>,
    has_tla: bool,
    async_parent_modules: Vec<SourceTextModule>,
    import_meta: Option<JsObject>,
    import_entries: Vec<ImportEntry>,
    local_export_entries: Vec<LocalExportEntry>,
    indirect_export_entries: Vec<IndirectExportEntry>,
    star_export_entries: Vec<Sym>,
}

impl std::fmt::Debug for Inner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceTextModuleData")
            .field("code", &"ModuleItemList")
            .field("status", &self.status)
            .field("requested_modules", &self.requested_modules)
            .field("loaded_modules", &self.loaded_modules)
            .field("has_tla", &self.has_tla)
            .field("async_parent_modules", &self.async_parent_modules)
            .field("import_meta", &self.import_meta)
            .field("import_entries", &self.import_entries)
            .field("local_export_entries", &self.local_export_entries)
            .field("indirect_export_entries", &self.indirect_export_entries)
            .field("star_export_entries", &self.star_export_entries)
            .finish()
    }
}

unsafe impl Trace for Inner {
    custom_trace!(this, {
        mark(&this.status);
        for module in this.loaded_modules.values() {
            mark(module);
        }
        mark(&this.async_parent_modules);
        mark(&this.import_meta);
    });
}

impl SourceTextModule {
    /// Creates a new `SourceTextModule` from a parsed `ModuleItemList`.
    ///
    /// Contains part of the abstract operation [`ParseModule`][parse].
    ///
    /// [parse]: https://tc39.es/ecma262/#sec-parsemodule
    pub(super) fn new(code: ModuleItemList) -> Self {
        // 3. Let requestedModules be the ModuleRequests of body.
        let requested_modules = code.requests();
        // 4. Let importEntries be ImportEntries of body.
        let import_entries = code.import_entries();

        // 5. Let importedBoundNames be ImportedLocalNames(importEntries).
        // Can be ignored because this is just a simple `Iter::map`

        // 6. Let indirectExportEntries be a new empty List.
        let mut indirect_export_entries = Vec::new();
        // 7. Let localExportEntries be a new empty List.
        let mut local_export_entries = Vec::new();
        // 8. Let starExportEntries be a new empty List.
        let mut star_export_entries = Vec::new();

        // 10. For each ExportEntry Record ee of exportEntries, do
        for ee in code.export_entries() {
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
                        //  3. Else,
                        //      a. NOTE: This is a re-export of a single name.
                        //      b. Append the ExportEntry Record { [[ModuleRequest]]: ie.[[ModuleRequest]],
                        //         [[ImportName]]: ie.[[ImportName]], [[LocalName]]: null,
                        //         [[ExportName]]: ee.[[ExportName]] } to indirectExportEntries.
                        indirect_export_entries.push(IndirectExportEntry::new(
                            module,
                            ReExportImportName::Name(import),
                            entry.export_name(),
                        ));
                    } else {
                        // i. If importedBoundNames does not contain ee.[[LocalName]], then
                        //     1. Append ee to localExportEntries.

                        //     2. If ie.[[ImportName]] is namespace-object, then
                        //         a. NOTE: This is a re-export of an imported module namespace object.
                        //         b. Append ee to localExportEntries.
                        local_export_entries.push(entry);
                    }
                }
                // b. Else if ee.[[ImportName]] is all-but-default, then
                ExportEntry::StarReExport { module_request } => {
                    //     i. Assert: ee.[[ExportName]] is null.
                    //     ii. Append ee to starExportEntries.
                    star_export_entries.push(module_request);
                }
                // c. Else,
                //     i. Append ee to indirectExportEntries.
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
        SourceTextModule(Gc::new(GcRefCell::new(Inner {
            code,
            requested_modules,
            has_tla,
            import_entries,
            local_export_entries,
            indirect_export_entries,
            star_export_entries,
            status: Status::Unlinked,
            loaded_modules: HashMap::default(),
            async_parent_modules: Vec::default(),
            import_meta: None,
        })))
    }

    /// Concrete method [`LoadRequestedModules ( [ hostDefined ] )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-LoadRequestedModules
    pub(super) fn load(module: &Module, context: &mut Context<'_>) -> JsPromise {
        // TODO: 1. If hostDefined is not present, let hostDefined be empty.
        // 2. Let pc be ! NewPromiseCapability(%Promise%).
        let pc = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("capability creation must always succeed when using the `%Promise%` intrinsic");

        // 4. Perform InnerModuleLoading(state, module).
        module.inner_load(
            // 3. Let state be the GraphLoadingState Record {
            //     [[IsLoading]]: true, [[PendingModulesCount]]: 1, [[Visited]]: « »,
            //     [[PromiseCapability]]: pc, [[HostDefined]]: hostDefined
            // }.
            &Rc::new(GraphLoadingState {
                capability: pc.clone(),
                loading: Cell::new(true),
                pending_modules: Cell::new(1),
                visited: RefCell::default(),
            }),
            context,
        );

        // 5. Return pc.[[Promise]].
        JsPromise::from_object(pc.promise().clone())
            .expect("promise created from the %Promise% intrinsic is always native")
    }

    /// Abstract operation [`InnerModuleLoading`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-InnerModuleLoading
    pub(super) fn inner_load(
        module: &Module,
        state: &Rc<GraphLoadingState>,
        context: &mut Context<'_>,
    ) {
        let ModuleKind::SourceText(src) = module.kind() else {
            unreachable!("must only be called for `SourceTextModule`s");
        };

        // 2. If module is a Cyclic Module Record, module.[[Status]] is new, and state.[[Visited]] does not contain
        //    module, then
        // a. Append module to state.[[Visited]].
        if matches!(src.borrow().status, Status::Unlinked)
            && state.visited.borrow_mut().insert(src.clone())
        {
            // b. Let requestedModulesCount be the number of elements in module.[[RequestedModules]].
            let requested = src.borrow().requested_modules.clone();
            // c. Set state.[[PendingModulesCount]] to state.[[PendingModulesCount]] + requestedModulesCount.
            state
                .pending_modules
                .set(state.pending_modules.get() + requested.len());
            // d. For each String required of module.[[RequestedModules]], do
            for required in requested {
                //     i. If module.[[LoadedModules]] contains a Record whose [[Specifier]] is required, then
                let loaded = src.borrow().loaded_modules.get(&required).cloned();
                if let Some(loaded) = loaded {
                    //         1. Let record be that Record.
                    //         2. Perform InnerModuleLoading(state, record.[[Module]]).
                    loaded.inner_load(state, context);
                } else {
                    //     ii. Else,
                    //         1. Perform HostLoadImportedModule(module, required, state.[[HostDefined]], state).
                    //         2. NOTE: HostLoadImportedModule will call FinishLoadingImportedModule, which re-enters
                    //            the graph loading process through ContinueModuleLoading.
                    let name_specifier: JsString = context
                        .interner()
                        .resolve_expect(required)
                        .into_common(false);
                    let src = src.clone();
                    let state = state.clone();
                    context.module_loader().load_imported_module(
                        Referrer::Module(module.clone()),
                        name_specifier,
                        Box::new(move |completion, ctx| {
                            if let Ok(loaded) = &completion {
                                let mut src = src.borrow_mut();
                                let entry = src
                                    .loaded_modules
                                    .entry(required)
                                    .or_insert_with(|| loaded.clone());
                                debug_assert_eq!(entry, loaded);
                            }

                            Module::resume_load(&state, completion, ctx);
                        }),
                        context,
                    );
                }
                //     iii. If state.[[IsLoading]] is false, return unused.
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
        export_star_set: &mut Vec<SourceTextModule>,
    ) -> FxHashSet<Sym> {
        // 1. Assert: module.[[Status]] is not new.
        // 2. If exportStarSet is not present, set exportStarSet to a new empty List.

        // 3. If exportStarSet contains module, then
        if export_star_set.contains(self) {
            //     a. Assert: We've reached the starting point of an export * circularity.
            //     b. Return a new empty List.
            return FxHashSet::default();
        }

        // 4. Append module to exportStarSet.
        export_star_set.push(self.clone());

        let module = self.borrow();
        // 5. Let exportedNames be a new empty List.
        let mut exported_names = FxHashSet::default();

        // 6. For each ExportEntry Record e of module.[[LocalExportEntries]], do
        for e in &module.local_export_entries {
            //     a. Assert: module provides the direct binding for this export.
            //     b. Append e.[[ExportName]] to exportedNames.
            exported_names.insert(e.export_name());
        }

        // 7. For each ExportEntry Record e of module.[[IndirectExportEntries]], do
        for e in &module.indirect_export_entries {
            //     a. Assert: module imports a specific binding for this export.
            //     b. Append e.[[ExportName]] to exportedNames.
            exported_names.insert(e.export_name());
        }

        // 8. For each ExportEntry Record e of module.[[StarExportEntries]], do
        for e in &module.star_export_entries {
            //     a. Let requestedModule be GetImportedModule(module, e.[[ModuleRequest]]).
            let requested_module = module.loaded_modules[e].clone();

            //     b. Let starNames be requestedModule.GetExportedNames(exportStarSet).
            //     c. For each element n of starNames, do
            for n in requested_module.get_exported_names(export_star_set) {
                //         i. If SameValue(n, "default") is false, then
                if n != Sym::DEFAULT {
                    //             1. If exportedNames does not contain n, then
                    //                 a. Append n to exportedNames.
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
        module: &Module,
        export_name: Sym,
        resolve_set: &mut FxHashSet<(Module, Sym)>,
    ) -> Result<ResolvedBinding, ResolveExportError> {
        // 1. Assert: module.[[Status]] is not new.
        // 2. If resolveSet is not present, set resolveSet to a new empty List.
        let ModuleKind::SourceText(src) = module.kind() else {
        unreachable!("must only be called for `SourceTextModule`s");
    };

        // 3. For each Record { [[Module]], [[ExportName]] } r of resolveSet, do
        //     a. If module and r.[[Module]] are the same Module Record and SameValue(exportName, r.[[ExportName]]) is true, then
        if resolve_set.contains(&(module.clone(), export_name)) {
            //         i. Assert: This is a circular import request.
            //         ii. Return null.
            return Err(ResolveExportError::NotFound);
        }

        // 4. Append the Record { [[Module]]: module, [[ExportName]]: exportName } to resolveSet.
        resolve_set.insert((module.clone(), export_name));
        let src = src.borrow();

        // 5. For each ExportEntry Record e of module.[[LocalExportEntries]], do
        for e in &src.local_export_entries {
            //     a. If SameValue(exportName, e.[[ExportName]]) is true, then
            if export_name == e.export_name() {
                //         i. Assert: module provides the direct binding for this export.
                //         ii. Return ResolvedBinding Record { [[Module]]: module, [[BindingName]]: e.[[LocalName]] }.
                return Ok(ResolvedBinding {
                    module: module.clone(),
                    binding_name: BindingName::Name(e.local_name()),
                });
            }
        }

        // 6. For each ExportEntry Record e of module.[[IndirectExportEntries]], do
        for e in &src.indirect_export_entries {
            //     a. If SameValue(exportName, e.[[ExportName]]) is true, then
            if export_name == e.export_name() {
                //         i. Let importedModule be GetImportedModule(module, e.[[ModuleRequest]]).
                let imported_module = &src.loaded_modules[&e.module_request()];
                return match e.import_name() {
                    //         ii. If e.[[ImportName]] is all, then
                    //             1. Assert: module does not provide the direct binding for this export.
                    //             2. Return ResolvedBinding Record { [[Module]]: importedModule, [[BindingName]]: namespace }.
                    ReExportImportName::Star => Ok(ResolvedBinding {
                        module: imported_module.clone(),
                        binding_name: BindingName::Namespace,
                    }),
                    //         iii. Else,
                    //             1. Assert: module imports a specific binding for this export.
                    //             2. Return importedModule.ResolveExport(e.[[ImportName]], resolveSet).
                    ReExportImportName::Name(_) => {
                        imported_module.resolve_export(export_name, resolve_set)
                    }
                };
            }
        }

        // 7. If SameValue(exportName, "default") is true, then
        if export_name == Sym::DEFAULT {
            //     a. Assert: A default export was not explicitly defined by this module.
            //     b. Return null.
            //     c. NOTE: A default export cannot be provided by an export * from "mod" declaration.
            return Err(ResolveExportError::NotFound);
        }

        // 8. Let starResolution be null.
        let mut star_resolution: Option<ResolvedBinding> = None;

        // 9. For each ExportEntry Record e of module.[[StarExportEntries]], do
        for e in &src.star_export_entries {
            //     a. Let importedModule be GetImportedModule(module, e.[[ModuleRequest]]).
            let imported_module = &src.loaded_modules[e];
            //     b. Let resolution be importedModule.ResolveExport(exportName, resolveSet).
            let resolution = match imported_module.resolve_export(export_name, resolve_set) {
                //     d. If resolution is not null, then
                Ok(resolution) => resolution,
                //     c. If resolution is ambiguous, return ambiguous.
                Err(e @ ResolveExportError::Ambiguous) => return Err(e),
                Err(ResolveExportError::NotFound) => continue,
            };

            //         i. Assert: resolution is a ResolvedBinding Record.
            if let Some(star_resolution) = &star_resolution {
                //         iii. Else,
                //             1. Assert: There is more than one * import that includes the requested name.
                //             2. If resolution.[[Module]] and starResolution.[[Module]] are not the same Module Record,
                //                return ambiguous.
                if resolution.module != star_resolution.module {
                    return Err(ResolveExportError::Ambiguous);
                }
                match (resolution.binding_name, star_resolution.binding_name) {
                    //             3. If resolution.[[BindingName]] is not starResolution.[[BindingName]] and either
                    //                resolution.[[BindingName]] or starResolution.[[BindingName]] is namespace,
                    //                return ambiguous.
                    (BindingName::Namespace, BindingName::Name(_))
                    | (BindingName::Name(_), BindingName::Namespace) => {
                        return Err(ResolveExportError::Ambiguous);
                    }
                    //             4. If resolution.[[BindingName]] is a String, starResolution.[[BindingName]] is a
                    //                String, and SameValue(resolution.[[BindingName]], starResolution.[[BindingName]])
                    //                is false, return ambiguous.
                    (BindingName::Name(res), BindingName::Name(star)) if res != star => {
                        return Err(ResolveExportError::Ambiguous);
                    }
                    _ => {}
                }
            } else {
                //         ii. If starResolution is null, then
                //             1. Set starResolution to resolution.
                star_resolution = Some(resolution);
            }
        }

        // 10. Return starResolution.
        star_resolution.ok_or(ResolveExportError::NotFound)
    }

    /// Concrete method [`Link ( )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-moduledeclarationlinking
    pub(super) fn link(module: &Module, context: &mut Context<'_>) -> JsResult<()> {
        let ModuleKind::SourceText(src) = module.kind() else {
            unreachable!("must only be called for `SourceTextModule`s");
        };

        // 1. Assert: module.[[Status]] is one of unlinked, linked, evaluating-async, or evaluated.
        debug_assert!(matches!(
            src.borrow().status,
            Status::Unlinked
                | Status::Linked { .. }
                | Status::EvaluatingAsync { .. }
                | Status::Evaluated { .. }
        ));

        // 2. Let stack be a new empty List.
        let mut stack = Vec::new();

        // 3. Let result be Completion(InnerModuleLinking(module, stack, 0)).
        // 4. If result is an abrupt completion, then
        if let Err(err) = Self::inner_link(module, &mut stack, 0, context) {
            //     a. For each Cyclic Module Record m of stack, do
            for m in stack {
                //         i. Assert: m.[[Status]] is linking.
                debug_assert!(matches!(m.borrow().status, Status::Linking { .. }));
                //         ii. Set m.[[Status]] to unlinked.
                m.borrow_mut().status = Status::Unlinked;
            }
            //     b. Assert: module.[[Status]] is unlinked.
            assert!(matches!(src.borrow().status, Status::Unlinked));
            //     c. Return ? result.
            return Err(err);
        }

        // 5. Assert: module.[[Status]] is one of linked, evaluating-async, or evaluated.
        debug_assert!(matches!(
            src.borrow().status,
            Status::Linked { .. } | Status::EvaluatingAsync { .. } | Status::Evaluated { .. }
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
        module: &Module,
        stack: &mut Vec<Self>,
        mut index: usize,
        context: &mut Context<'_>,
    ) -> JsResult<usize> {
        let ModuleKind::SourceText(src) = module.kind() else {
            unreachable!("must only be called for `SourceTextModule`s");
        };

        // 2. If module.[[Status]] is one of linking, linked, evaluating-async, or evaluated, then
        if matches!(
            src.borrow().status,
            Status::Linking { .. }
                | Status::PreLinked { .. }
                | Status::Linked { .. }
                | Status::EvaluatingAsync { .. }
                | Status::Evaluated { .. }
        ) {
            //     a. Return index.
            return Ok(index);
        }

        // 3. Assert: module.[[Status]] is unlinked.
        debug_assert!(matches!(src.borrow().status, Status::Unlinked));

        {
            let mut module = src.borrow_mut();
            // 4. Set module.[[Status]] to linking.
            // 5. Set module.[[DFSIndex]] to index.
            // 6. Set module.[[DFSAncestorIndex]] to index.
            module.status = Status::Linking {
                info: DfsInfo {
                    dfs_index: index,
                    dfs_ancestor_index: index,
                },
            };
        }

        // 7. Set index to index + 1.
        index += 1;

        // 8. Append module to stack.
        stack.push(src.clone());

        // 9. For each String required of module.[[RequestedModules]], do

        let requested = src.borrow().requested_modules.clone();
        for required in requested {
            //     a. Let requiredModule be GetImportedModule(module, required).
            let required_module = src.borrow().loaded_modules[&required].clone();

            //     b. Set index to ? InnerModuleLinking(requiredModule, stack, index).
            index = required_module.inner_link(stack, index, context)?;
            //     c. If requiredModule is a Cyclic Module Record, then
            if let ModuleKind::SourceText(required_module) = required_module.kind() {
                //         i. Assert: requiredModule.[[Status]] is one of linking, linked, evaluating-async, or evaluated.
                //         ii. Assert: requiredModule.[[Status]] is linking if and only if stack contains requiredModule.
                debug_assert!(match required_module.borrow().status {
                    Status::PreLinked { .. }
                    | Status::Linked { .. }
                    | Status::EvaluatingAsync { .. }
                    | Status::Evaluated { .. } => true,
                    Status::Linking { .. } if stack.contains(required_module) => true,
                    _ => false,
                });

                //         iii. If requiredModule.[[Status]] is linking, then
                let required_index = if let Status::Linking {
                    info:
                        DfsInfo {
                            dfs_ancestor_index, ..
                        },
                } = &required_module.borrow().status
                {
                    //             1. Set module.[[DFSAncestorIndex]] to min(module.[[DFSAncestorIndex]],
                    //                requiredModule.[[DFSAncestorIndex]]).

                    Some(*dfs_ancestor_index)
                } else {
                    None
                };

                if let Some(required_index) = required_index {
                    let mut module = src.borrow_mut();

                    let DfsInfo {
                        dfs_ancestor_index, ..
                    } = module
                        .status
                        .dfs_info_mut()
                        .expect("should be on the linking state");
                    *dfs_ancestor_index = usize::min(*dfs_ancestor_index, required_index);
                }
            }
        }
        // 10. Perform ? module.InitializeEnvironment().
        Self::initialize_environment(module, context)?;
        // 11. Assert: module occurs exactly once in stack.
        debug_assert_eq!(stack.iter().filter(|module| *module == src).count(), 1);
        // 12. Assert: module.[[DFSAncestorIndex]] ≤ module.[[DFSIndex]].
        debug_assert!({
            let DfsInfo {
                dfs_ancestor_index,
                dfs_index,
            } = src
                .borrow()
                .status
                .dfs_info()
                .copied()
                .expect("should be linking");
            dfs_ancestor_index <= dfs_index
        });

        let info = src.borrow().status.dfs_info().copied();
        match info {
            // 13. If module.[[DFSAncestorIndex]] = module.[[DFSIndex]], then

            //     a. Let done be false.
            //     b. Repeat, while done is false,
            Some(info) if info.dfs_ancestor_index == info.dfs_index => loop {
                //         i. Let requiredModule be the last element of stack.
                //         ii. Remove the last element of stack.
                //         iii. Assert: requiredModule is a Cyclic Module Record.
                let last = stack.pop().expect("should have at least one element");
                //         iv. Set requiredModule.[[Status]] to linked.
                last.borrow_mut()
                    .status
                    .transition(|current| match current {
                        Status::PreLinked { info, context } => Status::Linked { info, context },
                        _ => {
                            unreachable!(
                                "can only transition to `Linked` from the `PreLinked` state"
                            )
                        }
                    });

                //         v. If requiredModule and module are the same Module Record, set done to true.
                if &last == src {
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
    pub(super) fn evaluate(&self, context: &mut Context<'_>) -> JsPromise {
        // 1. Assert: This call to Evaluate is not happening at the same time as another call to Evaluate within the surrounding agent.
        let (module, promise) = {
            let this = self.borrow();
            match &this.status {
                Status::Unlinked
                | Status::Linking { .. }
                | Status::PreLinked { .. }
                | Status::Evaluating { .. } => {
                    unreachable!("2. Assert: module.[[Status]] is one of linked, evaluating-async, or evaluated.")
                }
                Status::Linked { .. } => (self.clone(), None),
                // 3. If module.[[Status]] is either evaluating-async or evaluated, set module to module.[[CycleRoot]].
                Status::EvaluatingAsync {
                    cycle_root,
                    top_level_capability,
                    ..
                }
                | Status::Evaluated {
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
        //     a. Return module.[[TopLevelCapability]].[[Promise]].
        if let Some(promise) = promise {
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
        let result = module.inner_evaluate(&mut stack, 0, Some(capability.clone()), context);

        match result {
            Ok(_) => {
                // 10. Else,
                //     a. Assert: module.[[Status]] is either evaluating-async or evaluated.
                assert!(match &module.borrow().status {
                    Status::EvaluatingAsync { .. } => true,
                    //     b. Assert: module.[[EvaluationError]] is empty.
                    Status::Evaluated { error, .. } if error.is_none() => true,
                    _ => false,
                });

                //     c. If module.[[AsyncEvaluation]] is false, then
                if matches!(&module.borrow().status, Status::Evaluated { .. }) {
                    //         i. Assert: module.[[Status]] is evaluated.
                    //         ii. Perform ! Call(capability.[[Resolve]], undefined, « undefined »).
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
                //     a. For each Cyclic Module Record m of stack, do
                for m in stack {
                    //         i. Assert: m.[[Status]] is evaluating.
                    //         ii. Set m.[[Status]] to evaluated.
                    //         iii. Set m.[[EvaluationError]] to result.
                    m.borrow_mut().status.
                    transition(|current| match current {
                        Status::Evaluating {
                            top_level_capability,
                            cycle_root,
                            ..
                        }
                        | Status::EvaluatingAsync {
                            top_level_capability,
                            cycle_root,
                                ..
                            } => Status::Evaluated {
                                top_level_capability,
                                cycle_root,
                                error: Some(err.clone()),
                            },
                            _ => panic!(
                                "can only transition to `Evaluated` from the `Evaluating` or `EvaluatingAsync states"
                            ),
                        });
                }
                //     b. Assert: module.[[Status]] is evaluated.
                //     c. Assert: module.[[EvaluationError]] is result.
                assert!(
                    matches!(&self.borrow().status, Status::Evaluated { error, .. } if error.is_some())
                );

                //     d. Perform ! Call(capability.[[Reject]], undefined, « result.[[Value]] »).
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
        stack: &mut Vec<Self>,
        mut index: usize,
        capability: Option<PromiseCapability>,
        context: &mut Context<'_>,
    ) -> JsResult<usize> {
        /// Gets the next evaluation index of an async module.
        ///
        /// Returns an error if there's no more available indices.
        fn get_async_eval_index() -> JsResult<usize> {
            thread_local! {
                static ASYNC_EVAL_QUEUE_INDEX: Cell<usize> = Cell::new(0);
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
        match &self.borrow_mut().status {
            // 3. If module.[[Status]] is evaluating, return index.
            Status::Evaluating { .. } | Status::EvaluatingAsync { .. } => return Ok(index),
            //     a. If module.[[EvaluationError]] is empty, return index.
            //     b. Otherwise, return ? module.[[EvaluationError]].
            Status::Evaluated { error, .. } => return error.clone().map_or(Ok(index), Err),
            Status::Linked { .. } => {
                // 4. Assert: module.[[Status]] is linked.
                // evaluate a linked module
            }
            _ => unreachable!(
                "2. Assert: module.[[Status]] is one of linked, evaluating-async, or evaluated."
            ),
        }

        let this = self.clone();
        // 5. Set module.[[Status]] to evaluating.
        // 6. Set module.[[DFSIndex]] to index.
        // 7. Set module.[[DFSAncestorIndex]] to index.
        // 8. Set module.[[PendingAsyncDependencies]] to 0.
        self.borrow_mut().status.transition(|status| match status {
            Status::Linked { context, .. } => Status::Evaluating {
                context,
                top_level_capability: capability,
                cycle_root: this,
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
        stack.push(self.clone());

        // 11. For each String required of module.[[RequestedModules]], do
        let requested = self.borrow().requested_modules.clone();
        for required in requested {
            //     a. Let requiredModule be GetImportedModule(module, required).
            let required_module = self.borrow().loaded_modules[&required].clone();
            //     b. Set index to ? InnerModuleEvaluation(requiredModule, stack, index).
            index = required_module.inner_evaluate(stack, index, context)?;

            //     c. If requiredModule is a Cyclic Module Record, then
            if let ModuleKind::SourceText(required_module) = required_module.kind() {
                //         i. Assert: requiredModule.[[Status]] is one of evaluating, evaluating-async, or evaluated.
                //         ii. Assert: requiredModule.[[Status]] is evaluating if and only if stack contains requiredModule.
                debug_assert!(match required_module.borrow().status {
                    Status::EvaluatingAsync { .. } | Status::Evaluated { .. } => true,
                    Status::Evaluating { .. } if stack.contains(required_module) => true,
                    _ => false,
                });

                let (required_module, async_eval, req_info) = match &required_module.borrow().status {
                    //         iii. If requiredModule.[[Status]] is evaluating, then
                    Status::Evaluating {
                        info,
                        async_eval_index,
                        ..
                    } => {
                        //             1. Set module.[[DFSAncestorIndex]] to min(module.[[DFSAncestorIndex]], requiredModule.[[DFSAncestorIndex]]).
                        (required_module.clone(), async_eval_index.is_some(), Some(*info))
                    }
                    //         iv. Else,
                    Status::EvaluatingAsync { cycle_root, .. }
                    | Status::Evaluated { cycle_root, .. } => {
                        //             1. Set requiredModule to requiredModule.[[CycleRoot]].
                        //             2. Assert: requiredModule.[[Status]] is either evaluating-async or evaluated.
                        match &cycle_root.borrow().status {
                            Status::EvaluatingAsync { .. } => (cycle_root.clone(), true, None),
                            //             3. If requiredModule.[[EvaluationError]] is not empty, return ? requiredModule.[[EvaluationError]].
                            Status::Evaluated { error: Some(error), .. } => return Err(error.clone()),
                            Status::Evaluated { .. } => (cycle_root.clone(), false, None),
                            _ => unreachable!("2. Assert: requiredModule.[[Status]] is either evaluating-async or evaluated."),
                        }
                    }
                    _ => unreachable!("i. Assert: requiredModule.[[Status]] is one of evaluating, evaluating-async, or evaluated."),
                };

                if let Some(req_info) = req_info {
                    let mut this = self.borrow_mut();
                    let info = this
                        .status
                        .dfs_info_mut()
                        .expect("self should still be in the evaluating state");
                    info.dfs_ancestor_index =
                        usize::min(info.dfs_ancestor_index, req_info.dfs_ancestor_index);
                }

                //         v. If requiredModule.[[AsyncEvaluation]] is true, then
                if async_eval {
                    //             1. Set module.[[PendingAsyncDependencies]] to module.[[PendingAsyncDependencies]] + 1.
                    pending_async_dependencies += 1;
                    //             2. Append module to requiredModule.[[AsyncParentModules]].
                    required_module
                        .borrow_mut()
                        .async_parent_modules
                        .push(self.clone());
                }
            }
        }

        // 12. If module.[[PendingAsyncDependencies]] > 0 or module.[[HasTLA]] is true, then
        if pending_async_dependencies > 0 || self.borrow().has_tla {
            //     a. Assert: module.[[AsyncEvaluation]] is false and was never previously set to true.
            {
                let Status::Evaluating { async_eval_index, .. } = &mut self.borrow_mut().status else {
                    unreachable!("self should still be in the evaluating state")
                };

                //     b. Set module.[[AsyncEvaluation]] to true.
                //     c. NOTE: The order in which module records have their [[AsyncEvaluation]] fields transition to true is significant. (See 16.2.1.5.3.4.)
                *async_eval_index = Some(get_async_eval_index()?);
            }

            //     d. If module.[[PendingAsyncDependencies]] = 0, perform ExecuteAsyncModule(module).
            if pending_async_dependencies == 0 {
                self.execute_async(context);
            }
        } else {
            // 13. Else,
            //     a. Perform ? module.ExecuteModule().
            self.execute(None, context)?;
        }

        let dfs_info = self.borrow().status.dfs_info().copied().expect(
            "haven't transitioned from the `Evaluating` state, so it should have its dfs info",
        );

        // 14. Assert: module occurs exactly once in stack.
        debug_assert_eq!(stack.iter().filter(|m| *m == self).count(), 1);
        // 15. Assert: module.[[DFSAncestorIndex]] ≤ module.[[DFSIndex]].
        assert!(dfs_info.dfs_ancestor_index <= dfs_info.dfs_index);

        // 16. If module.[[DFSAncestorIndex]] = module.[[DFSIndex]], then
        if dfs_info.dfs_ancestor_index == dfs_info.dfs_index {
            //     a. Let done be false.
            //     b. Repeat, while done is false,
            loop {
                //         i. Let requiredModule be the last element of stack.
                //         ii. Remove the last element of stack.
                let required_module = stack
                    .pop()
                    .expect("should at least have `self` in the stack");
                let is_self = self == &required_module;

                //         iii. Assert: requiredModule is a Cyclic Module Record.
                required_module.borrow_mut().status.transition(|current| match current {
                Status::Evaluating {
                            top_level_capability,
                            cycle_root,
                            async_eval_index,
                            context,
                            ..
                        } => if let Some(async_eval_index) = async_eval_index {
                            //         v. Otherwise, set requiredModule.[[Status]] to evaluating-async.
                            Status::EvaluatingAsync {
                                top_level_capability,
                                //         vii. Set requiredModule.[[CycleRoot]] to module.
                                cycle_root: if is_self {
                                    cycle_root
                                } else {
                                    self.clone()
                                },
                                async_eval_index,
                                pending_async_dependencies,
                                context
                            }
                        } else {
                            //         iv. If requiredModule.[[AsyncEvaluation]] is false, set requiredModule.[[Status]] to evaluated.
                            Status::Evaluated {
                                top_level_capability,
                                cycle_root: if is_self {
                                    cycle_root
                                } else {
                                    self.clone()
                                },
                                error: None,
                            }
                        }
                        _ => unreachable!(
                            "should only transition to `Evaluated` or `EvaluatingAsync` from the `Evaluating` state"
                        )
                    }
                );

                //         vi. If requiredModule and module are the same Module Record, set done to true.
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
    fn execute_async(&self, context: &mut Context<'_>) {
        // 1. Assert: module.[[Status]] is either evaluating or evaluating-async.
        debug_assert!(matches!(
            self.borrow().status,
            Status::Evaluating { .. } | Status::EvaluatingAsync { .. }
        ));
        // 2. Assert: module.[[HasTLA]] is true.
        debug_assert!(self.borrow().has_tla);

        // 3. Let capability be ! NewPromiseCapability(%Promise%).
        let capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("cannot fail for the %Promise% intrinsic");

        // 4. Let fulfilledClosure be a new Abstract Closure with no parameters that captures module and performs the following steps when called:
        // 5. Let onFulfilled be CreateBuiltinFunction(fulfilledClosure, 0, "", « »).
        let on_fulfilled = FunctionObjectBuilder::new(
            context,
            NativeFunction::from_copy_closure_with_captures(
                |_, _, module, context| {
                    //     a. Perform AsyncModuleExecutionFulfilled(module).
                    async_module_execution_fulfilled(module, context);
                    //     b. Return undefined.
                    Ok(JsValue::undefined())
                },
                self.clone(),
            ),
        )
        .build();

        // 6. Let rejectedClosure be a new Abstract Closure with parameters (error) that captures module and performs the following steps when called:
        // 7. Let onRejected be CreateBuiltinFunction(rejectedClosure, 0, "", « »).
        let on_rejected = FunctionObjectBuilder::new(
            context,
            NativeFunction::from_copy_closure_with_captures(
                |_, args, module, context| {
                    let error = JsError::from_opaque(args.get_or_undefined(0).clone());
                    //     a. Perform AsyncModuleExecutionRejected(module, error).
                    async_module_execution_rejected(module, &error, context);
                    //     b. Return undefined.
                    Ok(JsValue::undefined())
                },
                self.clone(),
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
        self.execute(Some(capability), context)
            .expect("async modules cannot directly throw");
    }

    /// Abstract operation [`GatherAvailableAncestors ( module, execList )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-gather-available-ancestors
    #[allow(clippy::mutable_key_type)]
    fn gather_available_ancestors(&self, exec_list: &mut FxHashSet<Self>) {
        // 1. For each Cyclic Module Record m of module.[[AsyncParentModules]], do
        for m in &self.borrow().async_parent_modules {
            //     a. If execList does not contain m and m.[[CycleRoot]].[[EvaluationError]] is empty, then
            // 2. Return unused.
            if !exec_list.contains(m)
                && m.borrow()
                    .status
                    .cycle_root()
                    .map_or(false, |cr| cr.borrow().status.evaluation_error().is_none())
            {
                let (deps, has_tla) = {
                    let m = &mut m.borrow_mut();
                    //         i. Assert: m.[[Status]] is evaluating-async.
                    //         ii. Assert: m.[[EvaluationError]] is empty.
                    //         iii. Assert: m.[[AsyncEvaluation]] is true.
                    let Status::EvaluatingAsync { pending_async_dependencies, .. } = &mut m.status else {
                        unreachable!("i. Assert: m.[[Status]] is evaluating-async.");
                    };
                    //         iv. Assert: m.[[PendingAsyncDependencies]] > 0.
                    assert!(*pending_async_dependencies > 0);

                    //         v. Set m.[[PendingAsyncDependencies]] to m.[[PendingAsyncDependencies]] - 1.
                    *pending_async_dependencies -= 1;
                    (*pending_async_dependencies, m.has_tla)
                };

                //         vi. If m.[[PendingAsyncDependencies]] = 0, then
                if deps == 0 {
                    //             1. Append m to execList.
                    exec_list.insert(m.clone());
                    //             2. If m.[[HasTLA]] is false, perform GatherAvailableAncestors(m, execList).
                    if !has_tla {
                        m.gather_available_ancestors(exec_list);
                    }
                }
            }
        }
    }

    /// Abstract operation [`InitializeEnvironment ( )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-source-text-module-record-initialize-environment
    fn initialize_environment(module: &Module, context: &mut Context<'_>) -> JsResult<()> {
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

        let ModuleKind::SourceText(src) = module.kind() else {
            unreachable!("must only be called for `SourceTextModule`s");
        };

        {
            let src = src.borrow();
            // 1. For each ExportEntry Record e of module.[[IndirectExportEntries]], do
            for e in &src.indirect_export_entries {
                //     a. Let resolution be module.ResolveExport(e.[[ExportName]]).
                module
                    .resolve_export(e.export_name(), &mut HashSet::default())
                    //     b. If resolution is either null or ambiguous, throw a SyntaxError exception.
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
                //     c. Assert: resolution is a ResolvedBinding Record.
            }
        }

        // 2. Assert: All named exports from module are resolvable.
        // 3. Let realm be module.[[Realm]].
        // 4. Assert: realm is not undefined.
        let mut realm = module.realm().clone();

        // 5. Let env be NewModuleEnvironment(realm.[[GlobalEnv]]).
        // 6. Set module.[[Environment]] to env.
        let global_env = realm.environment().clone();
        let global_compile_env = global_env.compile_env();
        let module_compile_env = Gc::new(GcRefCell::new(CompileTimeEnvironment::new(
            global_compile_env,
            true,
        )));

        let mut compiler =
            ByteCompiler::new(Sym::MAIN, true, false, module_compile_env.clone(), context);
        let mut imports = Vec::new();

        let codeblock = {
            // 7. For each ImportEntry Record in of module.[[ImportEntries]], do
            let src = src.borrow();
            for entry in &src.import_entries {
                //     a. Let importedModule be GetImportedModule(module, in.[[ModuleRequest]]).
                let imported_module = &src.loaded_modules[&entry.module_request()];

                if let ImportName::Name(name) = entry.import_name() {
                    //     c. Else,
                    //         i. Let resolution be importedModule.ResolveExport(in.[[ImportName]]).
                    let resolution =
                        imported_module
                            .resolve_export(name, &mut HashSet::default())
                            //         ii. If resolution is either null or ambiguous, throw a SyntaxError exception.
                            .map_err(|err| match err {
                                ResolveExportError::NotFound => JsNativeError::syntax()
                                    .with_message(format!(
                                        "could not find export `{}`",
                                        compiler.interner().resolve_expect(name)
                                    )),
                                ResolveExportError::Ambiguous => JsNativeError::syntax()
                                    .with_message(format!(
                                        "could not resolve ambiguous export `{}`",
                                        compiler.interner().resolve_expect(name)
                                    )),
                            })?;

                    //             2. Perform ! env.CreateImmutableBinding(in.[[LocalName]], true).
                    //             3. Perform ! env.InitializeBinding(in.[[LocalName]], namespace).
                    compiler.create_immutable_binding(entry.local_name(), true);
                    let locator = compiler.initialize_immutable_binding(entry.local_name());

                    if let BindingName::Name(_) = resolution.binding_name {
                        //     1. Perform env.CreateImportBinding(in.[[LocalName]], resolution.[[Module]],
                        //        resolution.[[BindingName]]).
                        //        deferred to initialization below
                        imports.push(ImportBinding::Single {
                            locator,
                            export_locator: resolution,
                        });
                    } else {
                        // 1. Let namespace be GetModuleNamespace(resolution.[[Module]]).
                        // deferred to initialization below
                        imports.push(ImportBinding::Namespace {
                            locator,
                            module: imported_module.clone(),
                        });
                    }
                } else {
                    //     b. If in.[[ImportName]] is namespace-object, then
                    //         ii. Perform ! env.CreateImmutableBinding(in.[[LocalName]], true).
                    compiler.create_immutable_binding(entry.local_name(), true);
                    //         iii. Perform ! env.InitializeBinding(in.[[LocalName]], namespace).
                    let locator = compiler.initialize_immutable_binding(entry.local_name());

                    //          i. Let namespace be GetModuleNamespace(importedModule).
                    //             deferred to initialization below
                    imports.push(ImportBinding::Namespace {
                        locator,
                        module: imported_module.clone(),
                    });
                }
            }

            // 18. Let code be module.[[ECMAScriptCode]].
            // 19. Let varDeclarations be the VarScopedDeclarations of code.
            let var_declarations = var_scoped_declarations(&src.code);
            // 20. Let declaredVarNames be a new empty List.
            let mut declared_var_names = Vec::new();
            // 21. For each element d of varDeclarations, do
            for var in var_declarations {
                //     a. For each element dn of the BoundNames of d, do
                for name in var.bound_names() {
                    //         i. If declaredVarNames does not contain dn, then
                    if !declared_var_names.contains(&name) {
                        //             1. Perform ! env.CreateMutableBinding(dn, false).
                        compiler.create_mutable_binding(name, false);
                        //             2. Perform ! env.InitializeBinding(dn, undefined).
                        let binding = compiler.initialize_mutable_binding(name, false);
                        let index = compiler.get_or_insert_binding(binding);
                        compiler.emit_opcode(Opcode::PushUndefined);
                        compiler.emit(Opcode::DefInitVar, &[index]);
                        //             3. Append dn to declaredVarNames.
                        declared_var_names.push(name);
                    }
                }
            }

            // 22. Let lexDeclarations be the LexicallyScopedDeclarations of code.
            // 23. Let privateEnv be null.
            let lex_declarations = lexically_scoped_declarations(&src.code);
            // 24. For each element d of lexDeclarations, do
            for declaration in lex_declarations {
                match &declaration {
                    //         i. If IsConstantDeclaration of d is true, then
                    Declaration::Lexical(LexicalDeclaration::Const(declaration)) => {
                        //     a. For each element dn of the BoundNames of d, do
                        for name in bound_names(declaration) {
                            //             1. Perform ! env.CreateImmutableBinding(dn, true).
                            compiler.create_immutable_binding(name, true);
                        }
                    }
                    //         ii. Else,
                    Declaration::Lexical(LexicalDeclaration::Let(declaration)) => {
                        //     a. For each element dn of the BoundNames of d, do
                        for name in bound_names(declaration) {
                            //             1. Perform ! env.CreateMutableBinding(dn, false).
                            compiler.create_mutable_binding(name, false);
                        }
                    }
                    //         iii. If d is either a FunctionDeclaration, a GeneratorDeclaration, an
                    //              AsyncFunctionDeclaration, or an AsyncGeneratorDeclaration, then
                    Declaration::Function(function) => {
                        //             1. Let fo be InstantiateFunctionObject of d with arguments env and privateEnv.
                        //             2. Perform ! env.InitializeBinding(dn, fo).
                        for name in bound_names(function) {
                            compiler.create_mutable_binding(name, false);
                        }
                        compiler.function(function.into(), NodeKind::Declaration, false);
                    }
                    Declaration::Generator(function) => {
                        //             1. Let fo be InstantiateFunctionObject of d with arguments env and privateEnv.
                        //             2. Perform ! env.InitializeBinding(dn, fo).
                        for name in bound_names(function) {
                            compiler.create_mutable_binding(name, false);
                        }
                        compiler.function(function.into(), NodeKind::Declaration, false);
                    }
                    Declaration::AsyncFunction(function) => {
                        //             1. Let fo be InstantiateFunctionObject of d with arguments env and privateEnv.
                        //             2. Perform ! env.InitializeBinding(dn, fo).
                        for name in bound_names(function) {
                            compiler.create_mutable_binding(name, false);
                        }
                        compiler.function(function.into(), NodeKind::Declaration, false);
                    }
                    Declaration::AsyncGenerator(function) => {
                        //             1. Let fo be InstantiateFunctionObject of d with arguments env and privateEnv.
                        //             2. Perform ! env.InitializeBinding(dn, fo).
                        for name in bound_names(function) {
                            compiler.create_mutable_binding(name, false);
                        }
                        compiler.function(function.into(), NodeKind::Declaration, false);
                    }
                    Declaration::Class(class) => {
                        //             1. Let fo be InstantiateFunctionObject of d with arguments env and privateEnv.
                        //             2. Perform ! env.InitializeBinding(dn, fo).
                        for name in bound_names(class) {
                            compiler.create_mutable_binding(name, false);
                        }
                    }
                }
            }

            compiler.compile_module_item_list(&src.code);

            Gc::new(compiler.finish())
        };

        // 8. Let moduleContext be a new ECMAScript code execution context.
        // 12. Set the ScriptOrModule of moduleContext to module.
        let mut envs = EnvironmentStack::new(global_env);
        envs.push_module(module_compile_env);

        // 13. Set the VariableEnvironment of moduleContext to module.[[Environment]].
        // 14. Set the LexicalEnvironment of moduleContext to module.[[Environment]].
        // 15. Set the PrivateEnvironment of moduleContext to null.
        std::mem::swap(&mut context.vm.environments, &mut envs);
        let stack = std::mem::take(&mut context.vm.stack);
        // 9. Set the Function of moduleContext to null.
        let active_function = context.vm.active_function.take();
        // 10. Assert: module.[[Realm]] is not undefined.
        // 11. Set the Realm of moduleContext to module.[[Realm]].
        context.swap_realm(&mut realm);
        // 17. Push moduleContext onto the execution context stack; moduleContext is now the running execution context.

        // deferred initialization of import bindings
        for import in imports {
            match import {
                ImportBinding::Namespace { locator, module } => {
                    //         i. Let namespace be GetModuleNamespace(importedModule).
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

        // 25. Remove moduleContext from the execution context stack.
        std::mem::swap(&mut context.vm.environments, &mut envs);
        context.vm.stack = stack;
        context.vm.active_function = active_function;
        context.swap_realm(&mut realm);

        debug_assert!(envs.current().as_declarative().is_some());
        *module.inner.environment.borrow_mut() = envs.current().as_declarative().cloned();

        // 16. Set module.[[Context]] to moduleContext.
        src.borrow_mut().status.transition(|state| match state {
            Status::Linking { info } => Status::PreLinked {
                info,
                context: SourceTextContext {
                    codeblock,
                    environments: envs,
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
        capability: Option<PromiseCapability>,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
        // 1. Let moduleContext be a new ECMAScript code execution context.
        let SourceTextContext {
            codeblock,
            mut environments,
            mut realm,
        } = match &self.borrow().status {
            Status::Evaluating { context, .. } | Status::EvaluatingAsync { context, .. } => {
                context.clone()
            }
            _ => unreachable!("`execute` should only be called for evaluating modules."),
        };

        let mut callframe = CallFrame::new(codeblock);
        callframe.promise_capability = capability;

        // 4. Set the ScriptOrModule of moduleContext to module.
        // 5. Assert: module has been linked and declarations in its module environment have been instantiated.
        // 6. Set the VariableEnvironment of moduleContext to module.[[Environment]].
        // 7. Set the LexicalEnvironment of moduleContext to module.[[Environment]].
        std::mem::swap(&mut context.vm.environments, &mut environments);
        let stack = std::mem::take(&mut context.vm.stack);
        // 2. Set the Function of moduleContext to null.
        let function = context.vm.active_function.take();
        // 3. Set the Realm of moduleContext to module.[[Realm]].
        context.swap_realm(&mut realm);
        // 8. Suspend the running execution context.
        context.vm.push_frame(callframe);

        // 9. If module.[[HasTLA]] is false, then
        //     a. Assert: capability is not present.
        //     b. Push moduleContext onto the execution context stack; moduleContext is now the running execution context.
        //     c. Let result be Completion(Evaluation of module.[[ECMAScriptCode]]).
        //     d. Suspend moduleContext and remove it from the execution context stack.
        //     e. Resume the context that is now on the top of the execution context stack as the running execution context.
        // 10. Else,
        //     a. Assert: capability is a PromiseCapability Record.
        //     b. Perform AsyncBlockStart(capability, module.[[ECMAScriptCode]], moduleContext).
        let result = context.run();

        std::mem::swap(&mut context.vm.environments, &mut environments);
        context.vm.stack = stack;
        context.vm.active_function = function;
        context.swap_realm(&mut realm);
        context.vm.pop_frame();

        //     f. If result is an abrupt completion, then
        if let CompletionRecord::Throw(err) = result {
            //         i. Return ? result.
            Err(err)
        } else {
            // 11. Return unused.
            Ok(())
        }
    }

    /// Borrows the inner data of the script module.
    #[track_caller]
    fn borrow(&self) -> GcRef<'_, Inner> {
        GcRefCell::borrow(&self.0)
    }

    /// Mutably borrows the inner data of the script module.
    #[track_caller]
    fn borrow_mut(&self) -> GcRefMut<'_, Inner> {
        GcRefCell::borrow_mut(&self.0)
    }
}

/// Abstract operation [`AsyncModuleExecutionFulfilled ( module )`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-async-module-execution-fulfilled
#[allow(clippy::mutable_key_type)]
fn async_module_execution_fulfilled(module: &SourceTextModule, context: &mut Context<'_>) {
    // 1. If module.[[Status]] is evaluated, then
    if let Status::Evaluated { error, .. } = &module.borrow().status {
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
    module
        .borrow_mut()
        .status
        .transition(|status| match status {
            Status::EvaluatingAsync {
                top_level_capability,
                cycle_root,
                ..
            } => Status::Evaluated {
                top_level_capability,
                cycle_root,
                error: None,
            },
            _ => unreachable!(),
        });

    // 7. If module.[[TopLevelCapability]] is not empty, then
    if let Some(cap) = module.borrow().status.top_level_capability() {
        //     a. Assert: module.[[CycleRoot]] is module.
        debug_assert_eq!(module.borrow().status.cycle_root(), Some(module));

        //     b. Perform ! Call(module.[[TopLevelCapability]].[[Resolve]], undefined, « undefined »).
        cap.resolve()
            .call(&JsValue::undefined(), &[], context)
            .expect("default `resolve` function cannot fail");
    }

    // 8. Let execList be a new empty List.
    let mut ancestors = FxHashSet::default();

    // 9. Perform GatherAvailableAncestors(module, execList).
    module.gather_available_ancestors(&mut ancestors);

    // 11. Assert: All elements of sortedExecList have their [[AsyncEvaluation]] field set to true, [[PendingAsyncDependencies]] field set to 0, and [[EvaluationError]] field set to empty.
    let mut ancestors = ancestors.into_iter().collect::<Vec<_>>();

    // 10. Let sortedExecList be a List whose elements are the elements of execList, in the order in which they had their [[AsyncEvaluation]] fields set to true in InnerModuleEvaluation.
    ancestors.sort_by_cached_key(|m| {
        let Status::EvaluatingAsync { async_eval_index, .. } = &m.borrow().status else {
        unreachable!("GatherAvailableAncestors: i. Assert: m.[[Status]] is evaluating-async.");
    };

        *async_eval_index
    });

    // 12. For each Cyclic Module Record m of sortedExecList, do
    for m in ancestors {
        //     a. If m.[[Status]] is evaluated, then
        if let Status::Evaluated { error, .. } = &m.borrow().status {
            //         i. Assert: m.[[EvaluationError]] is not empty.
            assert!(error.is_some());
            continue;
        }

        //     b. Else if m.[[HasTLA]] is true, then
        let has_tla = m.borrow().has_tla;
        if has_tla {
            //         i. Perform ExecuteAsyncModule(m).
            m.execute_async(context);
        } else {
            //     c. Else,
            //         i. Let result be m.ExecuteModule().
            let result = m.execute(None, context);

            //         ii. If result is an abrupt completion, then
            if let Err(e) = result {
                //             1. Perform AsyncModuleExecutionRejected(m, result.[[Value]]).
                async_module_execution_rejected(module, &e, context);
            } else {
                //         iii. Else,
                //             1. Set m.[[Status]] to evaluated.
                m.borrow_mut().status.transition(|status| match status {
                    Status::EvaluatingAsync {
                        top_level_capability,
                        cycle_root,
                        ..
                    } => Status::Evaluated {
                        top_level_capability,
                        cycle_root,
                        error: None,
                    },
                    _ => unreachable!(),
                });

                //             2. If m.[[TopLevelCapability]] is not empty, then
                if let Some(cap) = m.borrow().status.top_level_capability() {
                    //                 a. Assert: m.[[CycleRoot]] is m.
                    debug_assert_eq!(m.borrow().status.cycle_root(), Some(&m));

                    //                 b. Perform ! Call(m.[[TopLevelCapability]].[[Resolve]], undefined, « undefined »).
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
fn async_module_execution_rejected(
    module: &SourceTextModule,
    error: &JsError,
    context: &mut Context<'_>,
) {
    // 1. If module.[[Status]] is evaluated, then
    if let Status::Evaluated { error, .. } = &module.borrow().status {
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
    module
        .borrow_mut()
        .status
        .transition(|status| match status {
            Status::EvaluatingAsync {
                top_level_capability,
                cycle_root,
                ..
            } => Status::Evaluated {
                top_level_capability,
                cycle_root,
                error: Some(error.clone()),
            },
            _ => unreachable!(),
        });

    // 7. For each Cyclic Module Record m of module.[[AsyncParentModules]], do
    let module_b = module.borrow();
    for m in &module_b.async_parent_modules {
        //     a. Perform AsyncModuleExecutionRejected(m, error).
        async_module_execution_rejected(m, error, context);
    }

    // 8. If module.[[TopLevelCapability]] is not empty, then
    if let Some(cap) = module_b.status.top_level_capability() {
        //     a. Assert: module.[[CycleRoot]] is module.
        debug_assert_eq!(module_b.status.cycle_root(), Some(module));

        //     b. Perform ! Call(module.[[TopLevelCapability]].[[Reject]], undefined, « error »).
        cap.reject()
            .call(&JsValue::undefined(), &[error.to_opaque(context)], context)
            .expect("default `reject` function cannot fail");
    }
    // 9. Return unused.
}

impl PartialEq for SourceTextModule {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0.as_ref(), other.0.as_ref())
    }
}

impl Eq for SourceTextModule {}

impl Hash for SourceTextModule {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0.as_ref(), state);
    }
}
