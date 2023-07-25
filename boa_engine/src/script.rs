//! Boa's implementation of ECMAScript's Scripts.
//!
//! This module contains the [`Script`] type, which represents a [**Script Record**][script].
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-scripts
//! [script]: https://tc39.es/ecma262/#sec-script-records

use std::io::Read;

use boa_gc::{Finalize, Gc, GcRefCell, Trace};
use boa_interner::Sym;
use boa_parser::{Parser, Source};
use boa_profiler::Profiler;
use rustc_hash::FxHashMap;

use crate::{
    bytecompiler::ByteCompiler,
    realm::Realm,
    vm::{ActiveRunnable, CallFrame, CodeBlock},
    Context, JsResult, JsString, JsValue, Module,
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
            .field("host_defined", &self.inner.host_defined)
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
    host_defined: (),
}

impl Script {
    /// Gets the realm of this script.
    pub fn realm(&self) -> &Realm {
        &self.inner.realm
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
    pub fn parse<R: Read>(
        src: Source<'_, R>,
        realm: Option<Realm>,
        context: &mut Context<'_>,
    ) -> JsResult<Self> {
        let _timer = Profiler::global().start_event("Script parsing", "Main");
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
                host_defined: (),
            }),
        })
    }

    /// Compiles the codeblock of this script.
    ///
    /// This is a no-op if this has been called previously.
    pub fn codeblock(&self, context: &mut Context<'_>) -> JsResult<Gc<CodeBlock>> {
        let mut codeblock = self.inner.codeblock.borrow_mut();

        if let Some(codeblock) = &*codeblock {
            return Ok(codeblock.clone());
        };

        let _timer = Profiler::global().start_event("Script compilation", "Main");

        let mut compiler = ByteCompiler::new(
            Sym::MAIN,
            self.inner.source.strict(),
            false,
            self.inner.realm.environment().compile_env(),
            context,
        );
        // TODO: move to `Script::evaluate` to make this operation infallible.
        compiler.global_declaration_instantiation(&self.inner.source)?;
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
    pub fn evaluate(&self, context: &mut Context<'_>) -> JsResult<JsValue> {
        let _timer = Profiler::global().start_event("Execution", "Main");

        let codeblock = self.codeblock(context)?;

        let old_realm = context.enter_realm(self.inner.realm.clone());
        let active_function = context.vm.active_function.take();
        let old_active = context
            .vm
            .active_runnable
            .replace(ActiveRunnable::Script(self.clone()));
        let env_fp = context.vm.environments.len() as u32;
        context
            .vm
            .push_frame(CallFrame::new(codeblock).with_env_fp(env_fp));

        // TODO: Here should be https://tc39.es/ecma262/#sec-globaldeclarationinstantiation

        self.realm().resize_global_env();
        let record = context.run();
        context.vm.pop_frame();

        context.vm.active_function = active_function;
        context.vm.active_runnable = old_active;
        context.enter_realm(old_realm);

        context.clear_kept_objects();

        record.consume()
    }
}
