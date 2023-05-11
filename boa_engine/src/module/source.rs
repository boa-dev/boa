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

use super::{BindingName, ExportLocator, GraphLoadingState, Module, Referrer, ResolveExportError};

#[derive(Clone, Copy, Debug, Finalize)]
pub(super) struct DfsInfo {
    dfs_index: usize,
    dfs_ancestor_index: usize,
}

unsafe impl Trace for DfsInfo {
    empty_trace!();
}

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
    const fn dfs_info(&self) -> Option<&DfsInfo> {
        match self {
            Status::Unlinked | Status::EvaluatingAsync { .. } | Status::Evaluated { .. } => None,
            Status::Linking { info }
            | Status::PreLinked { info, .. }
            | Status::Linked { info, .. }
            | Status::Evaluating { info, .. } => Some(info),
        }
    }

    fn dfs_info_mut(&mut self) -> Option<&mut DfsInfo> {
        match self {
            Status::Unlinked | Status::EvaluatingAsync { .. } | Status::Evaluated { .. } => None,
            Status::Linking { info }
            | Status::PreLinked { info, .. }
            | Status::Linked { info, .. }
            | Status::Evaluating { info, .. } => Some(info),
        }
    }

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

    const fn evaluation_error(&self) -> Option<&JsError> {
        match &self {
            Status::Evaluated { error, .. } => error.as_ref(),
            _ => None,
        }
    }

    const fn cycle_root(&self) -> Option<&SourceTextModule> {
        match &self {
            Status::Evaluating { cycle_root, .. }
            | Status::EvaluatingAsync { cycle_root, .. }
            | Status::Evaluated { cycle_root, .. } => Some(cycle_root),
            _ => None,
        }
    }

    fn transition<F>(&mut self, f: F)
    where
        F: FnOnce(Status) -> Status,
    {
        *self = f(std::mem::take(self));
    }
}

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

#[derive(Finalize)]
struct SourceTextModuleData {
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

impl std::fmt::Debug for SourceTextModuleData {
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

unsafe impl Trace for SourceTextModuleData {
    custom_trace!(this, {
        mark(&this.status);
        for module in this.loaded_modules.values() {
            mark(module);
        }
        mark(&this.async_parent_modules);
        mark(&this.import_meta);
    });
}

///
#[derive(Clone, Trace, Finalize)]
pub struct SourceTextModule(Gc<GcRefCell<SourceTextModuleData>>);

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

impl SourceTextModule {
    /// Creates a new `SourceTextModule` from a parsed `ModuleItemList`.
    pub(super) fn new(code: ModuleItemList) -> Self {
        let requested_modules = code.requests();
        let import_entries = code.import_entries();

        let mut indirect_export_entries = Vec::new();
        let mut local_export_entries = Vec::new();
        let mut star_export_entries = Vec::new();
        for ee in code.export_entries() {
            match ee {
                ExportEntry::Ordinary(entry) => {
                    if let Some((module, import)) =
                        import_entries.iter().find_map(|ie| match ie.import_name() {
                            ImportName::Name(name) if ie.local_name() == entry.local_name() => {
                                Some((ie.module_request(), name))
                            }
                            _ => None,
                        })
                    {
                        indirect_export_entries.push(IndirectExportEntry::new(
                            module,
                            ReExportImportName::Name(import),
                            entry.export_name(),
                        ));
                    } else {
                        local_export_entries.push(entry);
                    }
                }
                ExportEntry::StarReExport { module_request } => {
                    star_export_entries.push(module_request);
                }
                ExportEntry::ReExport(entry) => indirect_export_entries.push(entry),
            }
        }

        let has_tla = contains(&code, ContainsSymbol::AwaitExpression);

        SourceTextModule(Gc::new(GcRefCell::new(SourceTextModuleData {
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

    /// Abstract operation [`LoadRequestedModules ( [ hostDefined ] )`][spec].
    ///
    /// Prepares the module for linking by loading all its module dependencies. Returns a `JsPromise`
    /// that will resolve when the loading process either completes or fails.
    ///
    /// [spec]: https://tc39.es/ecma262/#table-abstract-methods-of-module-records
    pub(super) fn load(module: &Module, context: &mut Context<'_>) -> JsPromise {
        let pc = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("capability creation must always succeed when using the `%Promise%` intrinsic");

        module.inner_load(
            &Rc::new(GraphLoadingState {
                capability: pc.clone(),
                loading: Cell::new(true),
                pending_modules: Cell::new(1),
                visited: RefCell::default(),
            }),
            context,
        );

        JsPromise::from_object(pc.promise().clone())
            .expect("promise created from the %Promise% intrinsic is always native")
    }

    pub(super) fn inner_load(
        module: &Module,
        state: &Rc<GraphLoadingState>,
        context: &mut Context<'_>,
    ) {
        let ModuleKind::SourceText(src) = module.kind() else {
            unreachable!("must only be called for `SourceTextModule`s");
        };

        if matches!(src.borrow().status, Status::Unlinked)
            && state.visited.borrow_mut().insert(src.clone())
        {
            let requested = src.borrow().requested_modules.clone();
            state
                .pending_modules
                .set(state.pending_modules.get() + requested.len());
            for required in requested {
                let loaded = src.borrow().loaded_modules.get(&required).cloned();
                if let Some(loaded) = loaded {
                    loaded.inner_load(state, context);
                } else {
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
                if !state.loading.get() {
                    return;
                }
            }
        }
    }

    pub(super) fn get_exported_names(
        &self,
        export_star_set: &mut Vec<SourceTextModule>,
    ) -> FxHashSet<Sym> {
        if export_star_set.contains(self) {
            return FxHashSet::default();
        }

        export_star_set.push(self.clone());

        let module = self.borrow();
        let mut exported_names = FxHashSet::default();

        for e in &module.local_export_entries {
            exported_names.insert(e.export_name());
        }

        for e in &module.indirect_export_entries {
            exported_names.insert(e.export_name());
        }

        for e in &module.star_export_entries {
            let requested_module = module.loaded_modules[e].clone();

            for n in requested_module.get_exported_names(export_star_set) {
                if n != Sym::DEFAULT {
                    exported_names.insert(n);
                }
            }
        }

        exported_names
    }

    #[allow(clippy::mutable_key_type)]
    pub(super) fn resolve_export(
        module: &Module,
        export_name: Sym,
        resolve_set: &mut FxHashSet<(Module, Sym)>,
    ) -> Result<ExportLocator, ResolveExportError> {
        let ModuleKind::SourceText(src) = module.kind() else {
            unreachable!("must only be called for `SourceTextModule`s");
        };

        if resolve_set.contains(&(module.clone(), export_name)) {
            return Err(ResolveExportError::NotFound);
        }

        resolve_set.insert((module.clone(), export_name));
        let src = src.borrow();

        for e in &src.local_export_entries {
            if export_name == e.export_name() {
                return Ok(ExportLocator {
                    module: module.clone(),
                    binding_name: BindingName::Name(e.local_name()),
                });
            }
        }

        for e in &src.indirect_export_entries {
            if export_name == e.export_name() {
                let imported_module = &src.loaded_modules[&e.module_request()];
                return match e.import_name() {
                    ReExportImportName::Star => Ok(ExportLocator {
                        module: imported_module.clone(),
                        binding_name: BindingName::Namespace,
                    }),
                    ReExportImportName::Name(_) => {
                        imported_module.resolve_export(export_name, resolve_set)
                    }
                };
            }
        }

        if export_name == Sym::DEFAULT {
            return Err(ResolveExportError::NotFound);
        }

        let mut star_resolution: Option<ExportLocator> = None;

        for e in &src.star_export_entries {
            let imported_module = &src.loaded_modules[e];
            let resolution = match imported_module.resolve_export(export_name, resolve_set) {
                Ok(resolution) => resolution,
                Err(e @ ResolveExportError::Ambiguous) => return Err(e),
                Err(ResolveExportError::NotFound) => continue,
            };

            if let Some(star_resolution) = &star_resolution {
                // 1. Assert: There is more than one * import that includes the requested name.
                if resolution.module != star_resolution.module {
                    return Err(ResolveExportError::Ambiguous);
                }
                match (resolution.binding_name, star_resolution.binding_name) {
                    (BindingName::Namespace, BindingName::Name(_))
                    | (BindingName::Name(_), BindingName::Namespace) => {
                        return Err(ResolveExportError::Ambiguous);
                    }
                    (BindingName::Name(res), BindingName::Name(star)) if res != star => {
                        return Err(ResolveExportError::Ambiguous);
                    }
                    _ => {}
                }
            } else {
                star_resolution = Some(resolution);
            }
        }

        star_resolution.ok_or(ResolveExportError::NotFound)
    }

    pub(super) fn link(module: &Module, context: &mut Context<'_>) -> JsResult<()> {
        let ModuleKind::SourceText(src) = module.kind() else {
            unreachable!("must only be called for `SourceTextModule`s");
        };
        debug_assert!(matches!(
            src.borrow().status,
            Status::Unlinked
                | Status::Linked { .. }
                | Status::EvaluatingAsync { .. }
                | Status::Evaluated { .. }
        ));

        let mut stack = Vec::new();

        if let Err(err) = Self::inner_link(module, &mut stack, 0, context) {
            for m in stack {
                debug_assert!(matches!(m.borrow().status, Status::Linking { .. }));
                m.borrow_mut().status = Status::Unlinked;
            }
            assert!(matches!(src.borrow().status, Status::Unlinked));
            return Err(err);
        }

        debug_assert!(matches!(
            src.borrow().status,
            Status::Linked { .. } | Status::EvaluatingAsync { .. } | Status::Evaluated { .. }
        ));
        assert!(stack.is_empty());

        Ok(())
    }

    pub(super) fn inner_link(
        module: &Module,
        stack: &mut Vec<Self>,
        mut index: usize,
        context: &mut Context<'_>,
    ) -> JsResult<usize> {
        let ModuleKind::SourceText(src) = module.kind() else {
            unreachable!("must only be called for `SourceTextModule`s");
        };
        if matches!(
            src.borrow().status,
            Status::Linking { .. }
                | Status::PreLinked { .. }
                | Status::Linked { .. }
                | Status::EvaluatingAsync { .. }
                | Status::Evaluated { .. }
        ) {
            return Ok(index);
        }

        debug_assert!(matches!(src.borrow().status, Status::Unlinked));

        {
            let mut module = src.borrow_mut();
            module.status = Status::Linking {
                info: DfsInfo {
                    dfs_index: index,
                    dfs_ancestor_index: index,
                },
            };
        }

        index += 1;

        stack.push(src.clone());

        let requested = src.borrow().requested_modules.clone();
        for required in requested {
            let required_module = src.borrow().loaded_modules[&required].clone();

            index = required_module.inner_link(stack, index, context)?;
            if let ModuleKind::SourceText(required_module) = required_module.kind() {
                debug_assert!(match required_module.borrow().status {
                    Status::PreLinked { .. }
                    | Status::Linked { .. }
                    | Status::EvaluatingAsync { .. }
                    | Status::Evaluated { .. } => true,
                    Status::Linking { .. } if stack.contains(required_module) => true,
                    _ => false,
                });

                let required_index = if let Status::Linking {
                    info:
                        DfsInfo {
                            dfs_ancestor_index, ..
                        },
                } = &required_module.borrow().status
                {
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
        Self::initialize_environment(module, context)?;
        debug_assert_eq!(stack.iter().filter(|module| *module == src).count(), 1);
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
            Some(info) if info.dfs_ancestor_index == info.dfs_index => loop {
                let last = stack.pop().expect("should have at least one element");
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
                if &last == src {
                    break;
                }
            },
            _ => {}
        }

        Ok(index)
    }

    pub(super) fn evaluate(&self, context: &mut Context<'_>) -> JsPromise {
        let (module, promise) = {
            let this = self.borrow();
            match &this.status {
                Status::Unlinked
                | Status::Linking { .. }
                | Status::PreLinked { .. }
                | Status::Evaluating { .. } => {
                    unreachable!()
                }
                Status::Linked { .. } => (self.clone(), None),
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

        if let Some(promise) = promise {
            return promise;
        }

        let mut stack = Vec::new();

        let capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("capability creation must always succeed when using the `%Promise%` intrinsic");

        let result = module.inner_evaluate(&mut stack, 0, Some(capability.clone()), context);

        match result {
            Ok(_) => {
                assert!(match &module.borrow().status {
                    Status::EvaluatingAsync { .. } => true,
                    Status::Evaluated { error, .. } if error.is_none() => true,
                    _ => false,
                });

                if matches!(&module.borrow().status, Status::Evaluated { .. }) {
                    capability
                        .resolve()
                        .call(&JsValue::undefined(), &[], context)
                        .expect("cannot fail for the default resolve function");
                }

                assert!(stack.is_empty());
            }
            Err(err) => {
                for m in stack {
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
                assert!(
                    matches!(&self.borrow().status, Status::Evaluated { error, .. } if error.is_some())
                );

                capability
                    .reject()
                    .call(&JsValue::undefined(), &[err.to_opaque(context)], context)
                    .expect("cannot fail for the default reject function");
            }
        }

        JsPromise::from_object(capability.promise().clone())
            .expect("promise created from the %Promise% intrinsic is always native")
    }

    pub(super) fn inner_evaluate(
        &self,
        stack: &mut Vec<Self>,
        mut index: usize,
        capability: Option<PromiseCapability>,
        context: &mut Context<'_>,
    ) -> JsResult<usize> {
        match &self.borrow_mut().status {
            Status::Evaluating { .. } | Status::EvaluatingAsync { .. } => return Ok(index),
            Status::Evaluated { error, .. } => return error.clone().map_or(Ok(index), Err),
            Status::Linked { .. } => {
                // evaluate a linked module
            }
            _ => unreachable!(
                "2. Assert: module.[[Status]] is one of linked, evaluating-async, or evaluated."
            ),
        }

        let this = self.clone();
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

        index += 1;

        let mut pending_async_dependencies = 0;
        stack.push(self.clone());

        let requested = self.borrow().requested_modules.clone();
        for required in requested {
            let required_module = self.borrow().loaded_modules[&required].clone();
            index = required_module.inner_evaluate(stack, index, context)?;

            if let ModuleKind::SourceText(required_module) = required_module.kind() {
                debug_assert!(match required_module.borrow().status {
                    Status::EvaluatingAsync { .. } | Status::Evaluated { .. } => true,
                    Status::Evaluating { .. } if stack.contains(required_module) => true,
                    _ => false,
                });

                let (required_module, async_eval, req_info) = match &required_module.borrow().status {
                    Status::Evaluating {
                        info,
                        async_eval_index,
                        ..
                    } => {
                        (required_module.clone(), async_eval_index.is_some(), Some(*info))
                    }
                    Status::EvaluatingAsync { cycle_root, .. }
                    | Status::Evaluated { cycle_root, .. } => {
                        match &cycle_root.borrow().status {
                            Status::EvaluatingAsync { .. } => (cycle_root.clone(), true, None),
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

                if async_eval {
                    pending_async_dependencies += 1;
                    required_module
                        .borrow_mut()
                        .async_parent_modules
                        .push(self.clone());
                }
            }
        }

        if pending_async_dependencies > 0 || self.borrow().has_tla {
            {
                let Status::Evaluating { async_eval_index, .. } = &mut self.borrow_mut().status else {
                    unreachable!("self should still be in the evaluating state")
                };

                *async_eval_index = Some(get_async_eval_index()?);
            }

            if pending_async_dependencies == 0 {
                self.execute_async(context);
            }
        } else {
            self.execute(None, context)?;
        }

        let dfs_info = self.borrow().status.dfs_info().copied().expect(
            "haven't transitioned from the `Evaluating` state, so it should have its dfs info",
        );

        debug_assert_eq!(stack.iter().filter(|m| *m == self).count(), 1);
        assert!(dfs_info.dfs_ancestor_index <= dfs_info.dfs_index);

        if dfs_info.dfs_ancestor_index == dfs_info.dfs_index {
            loop {
                let required_module = stack
                    .pop()
                    .expect("should at least have `self` in the stack");
                let is_self = self == &required_module;

                required_module.borrow_mut().status.transition(|current| match current {
                        Status::Evaluating {
                            top_level_capability,
                            cycle_root,
                            async_eval_index,
                            context,
                            ..
                        } => if let Some(async_eval_index) = async_eval_index {
                            Status::EvaluatingAsync {
                                top_level_capability,
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

                if is_self {
                    break;
                }
            }
        }

        Ok(index)
    }

    fn execute_async(&self, context: &mut Context<'_>) {
        assert!(matches!(
            self.borrow().status,
            Status::Evaluating { .. } | Status::EvaluatingAsync { .. }
        ));
        assert!(self.borrow().has_tla);

        let capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("cannot fail for the %Promise% intrinsic");

        let on_fulfilled = FunctionObjectBuilder::new(
            context,
            NativeFunction::from_copy_closure_with_captures(
                |_, _, module, context| {
                    async_module_execution_fulfilled(module, context);
                    Ok(JsValue::undefined())
                },
                self.clone(),
            ),
        )
        .build();

        let on_rejected = FunctionObjectBuilder::new(
            context,
            NativeFunction::from_copy_closure_with_captures(
                |_, args, module, context| {
                    let error = JsError::from_opaque(args.get_or_undefined(0).clone());
                    async_module_execution_rejected(module, &error, context);
                    Ok(JsValue::undefined())
                },
                self.clone(),
            ),
        )
        .build();

        Promise::perform_promise_then(
            capability.promise(),
            Some(on_fulfilled),
            Some(on_rejected),
            None,
            context,
        );

        self.execute(Some(capability), context)
            .expect("async modules cannot directly throw");
    }

    #[allow(clippy::mutable_key_type)]
    fn gather_available_ancestors(&self, exec_list: &mut FxHashSet<Self>) {
        for m in &self.borrow().async_parent_modules {
            if !exec_list.contains(m)
                && m.borrow()
                    .status
                    .cycle_root()
                    .map_or(false, |cr| cr.borrow().status.evaluation_error().is_none())
            {
                let (deps, has_tla) = {
                    let m = &mut m.borrow_mut();
                    let Status::EvaluatingAsync { pending_async_dependencies, .. } = &mut m.status else {
                        unreachable!("i. Assert: m.[[Status]] is evaluating-async.");
                    };

                    *pending_async_dependencies -= 1;
                    (*pending_async_dependencies, m.has_tla)
                };

                if deps == 0 {
                    exec_list.insert(m.clone());
                    if !has_tla {
                        m.gather_available_ancestors(exec_list);
                    }
                }
            }
        }
    }

    fn initialize_environment(module: &Module, context: &mut Context<'_>) -> JsResult<()> {
        #[derive(Debug)]
        enum ImportBinding {
            Namespace {
                locator: BindingLocator,
                module: Module,
            },
            Single {
                locator: BindingLocator,
                export_locator: ExportLocator,
            },
        }

        let ModuleKind::SourceText(src) = module.kind() else {
            unreachable!("must only be called for `SourceTextModule`s");
        };

        {
            let src = src.borrow();
            for e in &src.indirect_export_entries {
                module
                    .resolve_export(e.export_name(), &mut HashSet::default())
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
            }
        }

        let mut realm = module.realm().clone();
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
            let src = src.borrow();
            for entry in &src.import_entries {
                let imported_module = &src.loaded_modules[&entry.module_request()];

                if let ImportName::Name(name) = entry.import_name() {
                    let resolution =
                        imported_module
                            .resolve_export(name, &mut HashSet::default())
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

                    compiler.create_immutable_binding(entry.local_name(), true);
                    let locator = compiler.initialize_immutable_binding(entry.local_name());

                    if let BindingName::Name(_) = resolution.binding_name {
                        imports.push(ImportBinding::Single {
                            locator,
                            export_locator: resolution,
                        });
                    } else {
                        imports.push(ImportBinding::Namespace {
                            locator,
                            module: imported_module.clone(),
                        });
                    }
                } else {
                    compiler.create_immutable_binding(entry.local_name(), true);
                    let locator = compiler.initialize_immutable_binding(entry.local_name());
                    imports.push(ImportBinding::Namespace {
                        locator,
                        module: imported_module.clone(),
                    });
                }
            }

            let var_declarations = var_scoped_declarations(&src.code);
            let mut declared_var_names = Vec::new();
            for var in var_declarations {
                for name in var.bound_names() {
                    if !declared_var_names.contains(&name) {
                        compiler.create_mutable_binding(name, false);
                        let binding = compiler.initialize_mutable_binding(name, false);
                        let index = compiler.get_or_insert_binding(binding);
                        compiler.emit_opcode(Opcode::PushUndefined);
                        compiler.emit(Opcode::DefInitVar, &[index]);
                        declared_var_names.push(name);
                    }
                }
            }

            let lex_declarations = lexically_scoped_declarations(&src.code);
            for declaration in lex_declarations {
                match &declaration {
                    Declaration::Lexical(LexicalDeclaration::Const(declaration)) => {
                        for name in bound_names(declaration) {
                            compiler.create_immutable_binding(name, true);
                        }
                    }
                    Declaration::Lexical(LexicalDeclaration::Let(declaration)) => {
                        for name in bound_names(declaration) {
                            compiler.create_mutable_binding(name, false);
                        }
                    }
                    Declaration::Function(function) => {
                        for name in bound_names(function) {
                            compiler.create_mutable_binding(name, false);
                        }
                        compiler.function(function.into(), NodeKind::Declaration, false);
                    }
                    Declaration::Generator(function) => {
                        for name in bound_names(function) {
                            compiler.create_mutable_binding(name, false);
                        }
                        compiler.function(function.into(), NodeKind::Declaration, false);
                    }
                    Declaration::AsyncFunction(function) => {
                        for name in bound_names(function) {
                            compiler.create_mutable_binding(name, false);
                        }
                        compiler.function(function.into(), NodeKind::Declaration, false);
                    }
                    Declaration::AsyncGenerator(function) => {
                        for name in bound_names(function) {
                            compiler.create_mutable_binding(name, false);
                        }
                        compiler.function(function.into(), NodeKind::Declaration, false);
                    }
                    Declaration::Class(class) => {
                        for name in bound_names(class) {
                            compiler.create_mutable_binding(name, false);
                        }
                    }
                }
            }

            compiler.compile_module_item_list(&src.code);

            Gc::new(compiler.finish())
        };

        let mut envs = EnvironmentStack::new(global_env);
        envs.push_module(module_compile_env);

        std::mem::swap(&mut context.vm.environments, &mut envs);
        let stack = std::mem::take(&mut context.vm.stack);
        let active_function = context.vm.active_function.take();
        context.swap_realm(&mut realm);

        // initialize import bindings
        for import in imports {
            match import {
                ImportBinding::Namespace { locator, module } => {
                    let namespace = module.get_namespace(context);
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
                        let namespace = export_locator.module.get_namespace(context);
                        context.vm.environments.put_lexical_value(
                            locator.environment_index(),
                            locator.binding_index(),
                            namespace.into(),
                        );
                    }
                },
            }
        }

        std::mem::swap(&mut context.vm.environments, &mut envs);
        context.vm.stack = stack;
        context.vm.active_function = active_function;
        context.swap_realm(&mut realm);

        debug_assert!(envs.current().as_declarative().is_some());
        *module.inner.environment.borrow_mut() = envs.current().as_declarative().cloned();

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

        Ok(())
    }

    fn execute(
        &self,
        capability: Option<PromiseCapability>,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
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

        std::mem::swap(&mut context.vm.environments, &mut environments);
        let stack = std::mem::take(&mut context.vm.stack);
        let function = context.vm.active_function.take();
        context.swap_realm(&mut realm);
        context.vm.push_frame(callframe);

        let result = context.run();

        std::mem::swap(&mut context.vm.environments, &mut environments);
        context.vm.stack = stack;
        context.vm.active_function = function;
        context.swap_realm(&mut realm);
        context.vm.pop_frame();

        if let CompletionRecord::Throw(err) = result {
            Err(err)
        } else {
            Ok(())
        }
    }

    #[track_caller]
    fn borrow(&self) -> GcRef<'_, SourceTextModuleData> {
        GcRefCell::borrow(&self.0)
    }

    #[track_caller]
    fn borrow_mut(&self) -> GcRefMut<'_, SourceTextModuleData> {
        GcRefCell::borrow_mut(&self.0)
    }
}

/// [`AsyncModuleExecutionFulfilled ( module )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-async-module-execution-fulfilled
#[allow(clippy::mutable_key_type)]
fn async_module_execution_fulfilled(module: &SourceTextModule, context: &mut Context<'_>) {
    if let Status::Evaluated { error, .. } = &module.borrow().status {
        assert!(error.is_some());
        return;
    }

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

    if let Some(cap) = module.borrow().status.top_level_capability() {
        debug_assert_eq!(module.borrow().status.cycle_root(), Some(module));

        cap.resolve()
            .call(&JsValue::undefined(), &[], context)
            .expect("default `resolve` function cannot fail");
    }

    let mut ancestors = FxHashSet::default();

    module.gather_available_ancestors(&mut ancestors);

    let mut ancestors = ancestors.into_iter().collect::<Vec<_>>();

    ancestors.sort_by_cached_key(|m| {
        let Status::EvaluatingAsync { async_eval_index, .. } = &m.borrow().status else {
            unreachable!("GatherAvailableAncestors: i. Assert: m.[[Status]] is evaluating-async.");
        };

        *async_eval_index
    });

    for m in ancestors {
        if let Status::Evaluated { error, .. } = &m.borrow().status {
            assert!(error.is_some());
            continue;
        }

        let has_tla = m.borrow().has_tla;
        if has_tla {
            m.execute_async(context);
        } else {
            let result = m.execute(None, context);

            if let Err(e) = result {
                async_module_execution_rejected(module, &e, context);
            }

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

            if let Some(cap) = m.borrow().status.top_level_capability() {
                debug_assert_eq!(m.borrow().status.cycle_root(), Some(&m));

                cap.resolve()
                    .call(&JsValue::undefined(), &[], context)
                    .expect("default `resolve` function cannot fail");
            }
        }
    }
}

/// [`AsyncModuleExecutionRejected ( module, error )`][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-async-module-execution-rejected
fn async_module_execution_rejected(
    module: &SourceTextModule,
    error: &JsError,
    context: &mut Context<'_>,
) {
    if let Status::Evaluated { error, .. } = &module.borrow().status {
        assert!(error.is_some());
        return;
    }

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

    let module_b = module.borrow();
    for m in &module_b.async_parent_modules {
        async_module_execution_rejected(m, error, context);
    }

    if let Some(cap) = module_b.status.top_level_capability() {
        debug_assert_eq!(module_b.status.cycle_root(), Some(module));

        cap.reject()
            .call(&JsValue::undefined(), &[error.to_opaque(context)], context)
            .expect("default `reject` function cannot fail");
    }
}

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
