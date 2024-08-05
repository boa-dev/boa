//! Boa's implementation of ECMAScript's Scripts.
//!
//! This module contains the [`Script`] type, which represents a [**Script Record**][script].
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-scripts
//! [script]: https://tc39.es/ecma262/#sec-script-records

use std::path::{Path, PathBuf};

use rustc_hash::FxHashMap;

use boa_gc::{Finalize, Gc, GcRefCell, Trace};
use boa_parser::{source::ReadChar, Parser, Source};
use boa_profiler::Profiler;

use crate::{
    bytecompiler::{global_declaration_instantiation_context, ByteCompiler},
    js_string,
    realm::Realm,
    vm::{ActiveRunnable, CallFrame, CallFrameFlags, CodeBlock},
    Context, HostDefined, JsResult, JsString, JsValue, Module,
};

/// ECMAScript's [**Script Record**][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-script-records
#[derive(Clone, Trace, Finalize)]
pub struct Script {
    inner: Gc<Inner>,
}

impl std::fmt::Debug for Script {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Script")
            .field("realm", &self.inner.realm.addr())
            .field("code", &self.inner.source)
            .field("loaded_modules", &self.inner.loaded_modules)
            .finish()
    }
}

#[derive(Trace, Finalize)]
struct Inner {
    realm: Realm,
    #[unsafe_ignore_trace]
    source: boa_ast::Script,
    codeblock: GcRefCell<Option<Gc<CodeBlock>>>,
    loaded_modules: GcRefCell<FxHashMap<JsString, Module>>,
    host_defined: HostDefined,
    path: Option<PathBuf>,
}

impl Script {
    /// Gets the realm of this script.
    #[must_use]
    pub fn realm(&self) -> &Realm {
        &self.inner.realm
    }

    /// Returns the [`ECMAScript specification`][spec] defined [`\[\[HostDefined\]\]`][`HostDefined`] field of the [`Module`].
    ///
    /// [spec]: https://tc39.es/ecma262/#script-record
    #[must_use]
    pub fn host_defined(&self) -> &HostDefined {
        &self.inner.host_defined
    }

    /// Gets the loaded modules of this script.
    pub(crate) fn loaded_modules(&self) -> &GcRefCell<FxHashMap<JsString, Module>> {
        &self.inner.loaded_modules
    }

    /// Abstract operation [`ParseScript ( sourceText, realm, hostDefined )`][spec].
    ///
    /// Parses the provided `src` as an ECMAScript script, returning an error if parsing fails.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-parse-script
    pub fn parse<R: ReadChar>(
        src: Source<'_, R>,
        realm: Option<Realm>,
        context: &mut Context,
    ) -> JsResult<Self> {
        let _timer = Profiler::global().start_event("Script parsing", "Main");
        let path = src.path().map(Path::to_path_buf);
        let mut parser = Parser::new(src);
        parser.set_identifier(context.next_parser_identifier());
        if context.is_strict() {
            parser.set_strict();
        }
        let mut code = parser.parse_script(context.interner_mut())?;
        if !context.optimizer_options().is_empty() {
            context.optimize_statement_list(code.statements_mut());
        }

        Ok(Self {
            inner: Gc::new(Inner {
                realm: realm.unwrap_or_else(|| context.realm().clone()),
                source: code,
                codeblock: GcRefCell::default(),
                loaded_modules: GcRefCell::default(),
                host_defined: HostDefined::default(),
                path,
            }),
        })
    }

    /// Compiles the codeblock of this script.
    ///
    /// This is a no-op if this has been called previously.
    pub fn codeblock(&self, context: &mut Context) -> JsResult<Gc<CodeBlock>> {
        let mut codeblock = self.inner.codeblock.borrow_mut();

        if let Some(codeblock) = &*codeblock {
            return Ok(codeblock.clone());
        };

        let _timer = Profiler::global().start_event("Script compilation", "Main");

        let mut annex_b_function_names = Vec::new();

        global_declaration_instantiation_context(
            &mut annex_b_function_names,
            &self.inner.source,
            &self.inner.realm.environment().compile_env(),
            context,
        )?;

        let mut compiler = ByteCompiler::new(
            js_string!("<main>"),
            self.inner.source.strict(),
            false,
            self.inner.realm.environment().compile_env(),
            self.inner.realm.environment().compile_env(),
            false,
            false,
            context.interner_mut(),
            false,
        );

        #[cfg(feature = "annex-b")]
        {
            compiler.annex_b_function_names = annex_b_function_names;
        }

        // TODO: move to `Script::evaluate` to make this operation infallible.
        compiler.global_declaration_instantiation(
            &self.inner.source,
            &self.inner.realm.environment().compile_env(),
        );
        compiler.compile_statement_list(self.inner.source.statements(), true, false);

        let cb = Gc::new(compiler.finish());

        *codeblock = Some(cb.clone());

        Ok(cb)
    }

    /// Evaluates this script and returns its result.
    ///
    /// Note that this won't run any scheduled promise jobs; you need to call [`Context::run_jobs`]
    /// on the context or [`JobQueue::run_jobs`] on the provided queue to run them.
    ///
    /// [`JobQueue::run_jobs`]: crate::job::JobQueue::run_jobs
    pub fn evaluate(&self, context: &mut Context) -> JsResult<JsValue> {
        let _timer = Profiler::global().start_event("Execution", "Main");

        self.prepare_run(context)?;
        let record = context.run();

        context.vm.pop_frame();
        context.clear_kept_objects();

        record.consume()
    }

    /// Evaluates this script and returns its result, periodically yielding to the executor
    /// in order to avoid blocking the current thread.
    ///
    /// This uses an implementation defined amount of "clock cycles" that need to pass before
    /// execution is suspended. See [`Script::evaluate_async_with_budget`] if you want to also
    /// customize this parameter.
    #[allow(clippy::future_not_send)]
    pub async fn evaluate_async(&self, context: &mut Context) -> JsResult<JsValue> {
        self.evaluate_async_with_budget(context, 256).await
    }

    /// Evaluates this script and returns its result, yielding to the executor each time `budget`
    /// number of "clock cycles" pass.
    ///
    /// Note that "clock cycle" is in quotation marks because we can't determine exactly how many
    /// CPU clock cycles a VM instruction will take, but all instructions have a "cost" associated
    /// with them that depends on their individual complexity. We'd recommend benchmarking with
    /// different budget sizes in order to find the ideal yielding time for your application.
    #[allow(clippy::future_not_send)]
    pub async fn evaluate_async_with_budget(
        &self,
        context: &mut Context,
        budget: u32,
    ) -> JsResult<JsValue> {
        let _timer = Profiler::global().start_event("Async Execution", "Main");

        self.prepare_run(context)?;

        let record = context.run_async_with_budget(budget).await;

        context.vm.pop_frame();
        context.clear_kept_objects();

        record.consume()
    }

    fn prepare_run(&self, context: &mut Context) -> JsResult<()> {
        let codeblock = self.codeblock(context)?;

        let env_fp = context.vm.environments.len() as u32;
        context.vm.push_frame_with_stack(
            CallFrame::new(
                codeblock,
                Some(ActiveRunnable::Script(self.clone())),
                context.vm.environments.clone(),
                self.inner.realm.clone(),
            )
            .with_env_fp(env_fp)
            .with_flags(CallFrameFlags::EXIT_EARLY),
            JsValue::undefined(),
            JsValue::null(),
        );

        // TODO: Here should be https://tc39.es/ecma262/#sec-globaldeclarationinstantiation

        self.realm().resize_global_env();

        Ok(())
    }

    pub(super) fn path(&self) -> Option<&Path> {
        self.inner.path.as_deref()
    }
}
