use std::rc::Rc;

use boa_ast::expression::Identifier;
use boa_gc::{Finalize, Gc, GcRefCell, Trace, WeakGc};
use boa_interner::Sym;
use rustc_hash::FxHashSet;

use crate::{
    builtins::promise::ResolvingFunctions,
    bytecompiler::ByteCompiler,
    environments::{CompileTimeEnvironment, EnvironmentStack},
    object::JsPromise,
    vm::{ActiveRunnable, CallFrame, CodeBlock},
    Context, JsNativeError, JsResult, JsString, JsValue, Module,
};

use super::{BindingName, ModuleRepr, ResolveExportError, ResolvedBinding};

trait TraceableCallback: Trace {
    fn call(&self, module: &SyntheticModule, context: &mut Context<'_>) -> JsResult<()>;
}

#[derive(Trace, Finalize)]
struct Callback<F, T>
where
    F: Fn(&SyntheticModule, &T, &mut Context<'_>) -> JsResult<()>,
    T: Trace,
{
    // SAFETY: `SyntheticModuleInitializer`'s safe API ensures only `Copy` closures are stored; its unsafe API,
    // on the other hand, explains the invariants to hold in order for this to be safe, shifting
    // the responsibility to the caller.
    #[unsafe_ignore_trace]
    f: F,
    captures: T,
}

impl<F, T> TraceableCallback for Callback<F, T>
where
    F: Fn(&SyntheticModule, &T, &mut Context<'_>) -> JsResult<()>,
    T: Trace,
{
    fn call(&self, module: &SyntheticModule, context: &mut Context<'_>) -> JsResult<()> {
        (self.f)(module, &self.captures, context)
    }
}

/// The initializing steps of a [`SyntheticModule`].
///
/// # Caveats
///
/// By limitations of the Rust language, the garbage collector currently cannot inspect closures
/// in order to trace their captured variables. This means that only [`Copy`] closures are 100% safe
/// to use. All other closures can also be stored in a `NativeFunction`, albeit by using an `unsafe`
/// API, but note that passing closures implicitly capturing traceable types could cause
/// **Undefined Behaviour**.
#[derive(Clone, Trace, Finalize)]
pub struct SyntheticModuleInitializer {
    inner: Gc<dyn TraceableCallback>,
}

impl std::fmt::Debug for SyntheticModuleInitializer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleInitializer").finish_non_exhaustive()
    }
}

impl SyntheticModuleInitializer {
    /// Creates a `SyntheticModuleInitializer` from a [`Copy`] closure.
    pub fn from_copy_closure<F>(closure: F) -> Self
    where
        F: Fn(&SyntheticModule, &mut Context<'_>) -> JsResult<()> + Copy + 'static,
    {
        // SAFETY: The `Copy` bound ensures there are no traceable types inside the closure.
        unsafe { Self::from_closure(closure) }
    }

    /// Creates a `SyntheticModuleInitializer` from a [`Copy`] closure and a list of traceable captures.
    pub fn from_copy_closure_with_captures<F, T>(closure: F, captures: T) -> Self
    where
        F: Fn(&SyntheticModule, &T, &mut Context<'_>) -> JsResult<()> + Copy + 'static,
        T: Trace + 'static,
    {
        // SAFETY: The `Copy` bound ensures there are no traceable types inside the closure.
        unsafe { Self::from_closure_with_captures(closure, captures) }
    }

    /// Creates a new `SyntheticModuleInitializer` from a closure.
    ///
    /// # Safety
    ///
    /// Passing a closure that contains a captured variable that needs to be traced by the garbage
    /// collector could cause an use after free, memory corruption or other kinds of **Undefined
    /// Behaviour**. See <https://github.com/Manishearth/rust-gc/issues/50> for a technical explanation
    /// on why that is the case.
    pub unsafe fn from_closure<F>(closure: F) -> Self
    where
        F: Fn(&SyntheticModule, &mut Context<'_>) -> JsResult<()> + 'static,
    {
        // SAFETY: The caller must ensure the invariants of the closure hold.
        unsafe {
            Self::from_closure_with_captures(move |module, _, context| closure(module, context), ())
        }
    }

    /// Create a new `SyntheticModuleInitializer` from a closure and a list of traceable captures.
    ///
    /// # Safety
    ///
    /// Passing a closure that contains a captured variable that needs to be traced by the garbage
    /// collector could cause an use after free, memory corruption or other kinds of **Undefined
    /// Behaviour**. See <https://github.com/Manishearth/rust-gc/issues/50> for a technical explanation
    /// on why that is the case.
    pub unsafe fn from_closure_with_captures<F, T>(closure: F, captures: T) -> Self
    where
        F: Fn(&SyntheticModule, &T, &mut Context<'_>) -> JsResult<()> + 'static,
        T: Trace + 'static,
    {
        // Hopefully, this unsafe operation will be replaced by the `CoerceUnsized` API in the
        // future: https://github.com/rust-lang/rust/issues/18598
        let ptr = Gc::into_raw(Gc::new(Callback {
            f: closure,
            captures,
        }));

        // SAFETY: The pointer returned by `into_raw` is only used to coerce to a trait object,
        // meaning this is safe.
        unsafe {
            Self {
                inner: Gc::from_raw(ptr),
            }
        }
    }

    /// Calls this `SyntheticModuleInitializer`, forwarding the arguments to the corresponding function.
    #[inline]
    pub fn call(&self, module: &SyntheticModule, context: &mut Context<'_>) -> JsResult<()> {
        self.inner.call(module, context)
    }
}

/// ECMAScript's [**Synthetic Module Records**][spec].
///
/// [spec]: https://tc39.es/proposal-json-modules/#sec-synthetic-module-records
#[derive(Clone, Trace, Finalize)]
pub struct SyntheticModule {
    inner: Gc<Inner>,
}

impl std::fmt::Debug for SyntheticModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyntheticModule")
            .field("export_names", &self.inner.export_names)
            .field("eval_steps", &self.inner.eval_steps)
            .finish_non_exhaustive()
    }
}

#[derive(Trace, Finalize)]
struct Inner {
    parent: WeakGc<ModuleRepr>,
    #[unsafe_ignore_trace]
    export_names: FxHashSet<Sym>,
    eval_context: GcRefCell<Option<(EnvironmentStack, Gc<CodeBlock>)>>,
    eval_steps: SyntheticModuleInitializer,
}

impl SyntheticModule {
    /// Gets the parent module of this source module.
    fn parent(&self) -> Module {
        Module {
            inner: self
                .inner
                .parent
                .upgrade()
                .expect("parent module must be live"),
        }
    }

    /// Creates a new synthetic module.
    pub(super) fn new(
        names: FxHashSet<Sym>,
        eval_steps: SyntheticModuleInitializer,
        parent: WeakGc<ModuleRepr>,
    ) -> Self {
        Self {
            inner: Gc::new(Inner {
                parent,
                export_names: names,
                eval_steps,
                eval_context: GcRefCell::default(),
            }),
        }
    }

    /// Concrete method [`LoadRequestedModules ( )`][spec].
    ///
    /// [spec]: https://tc39.es/proposal-json-modules/#sec-smr-LoadRequestedModules
    pub(super) fn load(context: &mut Context<'_>) -> JsPromise {
        // 1. Return ! PromiseResolve(%Promise%, undefined).
        JsPromise::resolve(JsValue::undefined(), context)
            .expect("creating a promise from the %Promise% constructor must not fail")
    }

    /// Concrete method [`GetExportedNames ( [ exportStarSet ] )`][spec].
    ///
    /// [spec]: https://tc39.es/proposal-json-modules/#sec-smr-getexportednames
    pub(super) fn get_exported_names(&self) -> FxHashSet<Sym> {
        // 1. Return module.[[ExportNames]].
        self.inner.export_names.clone()
    }

    /// Concrete method [`ResolveExport ( exportName )`][spec]
    ///
    /// [spec]: https://tc39.es/proposal-json-modules/#sec-smr-resolveexport
    #[allow(clippy::mutable_key_type)]
    pub(super) fn resolve_export(
        &self,
        export_name: Sym,
    ) -> Result<ResolvedBinding, ResolveExportError> {
        if self.inner.export_names.contains(&export_name) {
            // 2. Return ResolvedBinding Record { [[Module]]: module, [[BindingName]]: exportName }.
            Ok(ResolvedBinding {
                module: self.parent(),
                binding_name: BindingName::Name(Identifier::new(export_name)),
            })
        } else {
            // 1. If module.[[ExportNames]] does not contain exportName, return null.
            Err(ResolveExportError::NotFound)
        }
    }

    /// Concrete method [`Link ( )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-moduledeclarationlinking
    pub(super) fn link(&self, context: &mut Context<'_>) {
        let parent = self.parent();
        // 1. Let realm be module.[[Realm]].
        // 2. Let env be NewModuleEnvironment(realm.[[GlobalEnv]]).
        // 3. Set module.[[Environment]] to env.
        let global_env = parent.realm().environment().clone();
        let global_compile_env = global_env.compile_env();
        let module_compile_env = Rc::new(CompileTimeEnvironment::new(global_compile_env, true));

        // TODO: A bit of a hack to be able to pass the currently active runnable without an
        // available codeblock to execute.
        let compiler = ByteCompiler::new(
            Sym::MAIN,
            true,
            false,
            module_compile_env.clone(),
            module_compile_env.clone(),
            context,
        );

        // 4. For each String exportName in module.[[ExportNames]], do
        let exports = self
            .inner
            .export_names
            .iter()
            .map(|name| {
                let ident = Identifier::new(*name);
                //     a. Perform ! env.CreateMutableBinding(exportName, false).
                module_compile_env.create_mutable_binding(ident, false)
            })
            .collect::<Vec<_>>();

        let cb = Gc::new(compiler.finish());

        let mut envs = EnvironmentStack::new(global_env);
        envs.push_module(module_compile_env);

        for locator in exports {
            //     b. Perform ! env.InitializeBinding(exportName, undefined).
            envs.put_lexical_value(
                locator.environment_index(),
                locator.binding_index(),
                JsValue::undefined(),
            );
        }

        *parent.inner.environment.borrow_mut() = envs.current().as_declarative().cloned();

        *self.inner.eval_context.borrow_mut() = Some((envs, cb));

        // 5. Return unused.
    }

    /// Concrete method [`Evaluate ( )`][spec].
    ///
    /// [spec]: https://tc39.es/proposal-json-modules/#sec-smr-Evaluate
    pub(super) fn evaluate(&self, context: &mut Context<'_>) -> JsPromise {
        // 1. Let moduleContext be a new ECMAScript code execution context.

        let parent = self.parent();
        let mut realm = parent.realm().clone();
        let (mut environments, codeblock) = self
            .inner
            .eval_context
            .borrow()
            .clone()
            .expect("should have been initialized on `link`");

        let env_fp = environments.len() as u32;
        let callframe = CallFrame::new(
            codeblock,
            // 4. Set the ScriptOrModule of moduleContext to module.
            Some(ActiveRunnable::Module(parent)),
            // 2. Set the Function of moduleContext to null.
            None,
        )
        .with_env_fp(env_fp);

        // 5. Set the VariableEnvironment of moduleContext to module.[[Environment]].
        // 6. Set the LexicalEnvironment of moduleContext to module.[[Environment]].
        std::mem::swap(&mut context.vm.environments, &mut environments);
        // 3. Set the Realm of moduleContext to module.[[Realm]].
        context.swap_realm(&mut realm);
        // 7. Suspend the currently running execution context.
        // 8. Push moduleContext on to the execution context stack; moduleContext is now the running execution context.
        context.vm.push_frame(callframe);

        // 9. Let steps be module.[[EvaluationSteps]].
        // 10. Let result be Completion(steps(module)).
        let result = self.inner.eval_steps.call(self, context);

        // 11. Suspend moduleContext and remove it from the execution context stack.
        // 12. Resume the context that is now on the top of the execution context stack as the running execution context.
        std::mem::swap(&mut context.vm.environments, &mut environments);
        context.swap_realm(&mut realm);
        context.vm.pop_frame();

        // 13. Let pc be ! NewPromiseCapability(%Promise%).
        let (promise, ResolvingFunctions { resolve, reject }) = JsPromise::new_pending(context);

        match result {
            // 15. Perform ! pc.[[Resolve]](result).
            Ok(()) => resolve.call(&JsValue::undefined(), &[], context),
            // 14. IfAbruptRejectPromise(result, pc).
            Err(err) => reject.call(&JsValue::undefined(), &[err.to_opaque(context)], context),
        }
        .expect("default resolving functions cannot throw");

        // 16. Return pc.[[Promise]].
        promise
    }

    /// Abstract operation [`SetSyntheticModuleExport ( module, exportName, exportValue )`][spec].
    ///
    /// Sets or changes the exported value for `exportName` in the synthetic module.
    ///
    /// # Note
    ///
    /// The default export corresponds to the name `"default"`, but note that it needs to
    /// be passed to the list of exported names in [`Module::synthetic`] beforehand.
    ///
    /// [spec]: https://tc39.es/proposal-json-modules/#sec-createsyntheticmodule
    pub fn set_export(
        &self,
        export_name: &JsString,
        export_value: JsValue,
        context: &mut Context<'_>,
    ) -> JsResult<()> {
        let identifier = context.interner_mut().get_or_intern(&**export_name);
        let identifier = Identifier::new(identifier);

        let environment = self
            .parent()
            .environment()
            .expect("this must be initialized before evaluating");
        let locator = environment
            .compile_env()
            .get_binding(identifier)
            .ok_or_else(|| {
                JsNativeError::reference().with_message(format!(
                    "cannot set name `{}` which was not included in the list of exports",
                    export_name.to_std_string_escaped()
                ))
            })?;
        environment.set(locator.binding_index(), export_value);

        Ok(())
    }
}
