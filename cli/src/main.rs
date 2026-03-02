//! A CLI implementation for `boa_engine` that comes complete with file execution and a REPL.
#![doc = include_str!("../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo_black.svg"
)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![allow(clippy::print_stdout, clippy::print_stderr)]

mod debug;
mod helper;
mod logger;

use crate::logger::SharedExternalPrinterLogger;
use boa_engine::context::time::JsInstant;
use boa_engine::error::JsErasedError;
use boa_engine::job::{GenericJob, TimeoutJob};
use boa_engine::{
    Context, JsError, JsResult, Source,
    builtins::promise::PromiseState,
    context::ContextBuilder,
    job::{Job, JobExecutor, NativeAsyncJob, PromiseJob},
    module::{Module, SimpleModuleLoader},
    optimizer::OptimizerOptions,
    script::Script,
    vm::flowgraph::{Direction, Graph},
};
use boa_parser::source::ReadChar;
use clap::{Parser, ValueEnum, ValueHint};
use color_eyre::{
    Result, Section,
    eyre::{WrapErr, eyre},
};
use colored::Colorize;
use debug::init_boa_debug_object;
use futures_concurrency::future::FutureGroup;
use futures_lite::{StreamExt, future};
use rustyline::{EditMode, Editor, config::Config, error::ReadlineError};
use std::collections::BTreeMap;
use std::mem;
use std::sync::mpsc::{Sender, TryRecvError};
use std::time::{Duration, Instant};
use std::{
    cell::RefCell,
    collections::VecDeque,
    fs::OpenOptions,
    io::{self, IsTerminal, Read},
    path::{Path, PathBuf},
    rc::Rc,
    thread,
};

// ----

#[cfg(all(
    target_arch = "x86_64",
    target_os = "linux",
    target_env = "gnu",
    feature = "dhat"
))]
use jemallocator as _;

#[cfg(all(
    target_arch = "x86_64",
    target_os = "linux",
    target_env = "gnu",
    feature = "fast-allocator",
    not(feature = "dhat")
))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(all(target_os = "windows", feature = "dhat"))]
use mimalloc_safe as _;

#[cfg(all(
    target_os = "windows",
    feature = "fast-allocator",
    not(feature = "dhat")
))]
#[global_allocator]
static ALLOC: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;

#[cfg(feature = "dhat")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// CLI configuration for Boa.
static CLI_HISTORY: &str = ".boa_history";

// Added #[allow(clippy::option_option)] because to StructOpt an Option<Option<T>>
// is an optional argument that optionally takes a value ([--opt=[val]]).
// https://docs.rs/structopt/0.3.11/structopt/#type-magic
#[derive(Debug, Parser)]
#[command(author, version, about, name = "boa")]
#[allow(clippy::struct_excessive_bools)] // NOTE: Allow having more than 3 bools in struct
struct Opt {
    /// The JavaScript file(s) to be evaluated.
    #[arg(name = "FILE", value_hint = ValueHint::FilePath)]
    files: Vec<PathBuf>,

    /// Run in strict mode.
    #[arg(long)]
    strict: bool,

    /// Dump the AST to stdout with the given format.
    #[arg(
        long,
        short = 'a',
        value_name = "FORMAT",
        ignore_case = true,
        value_enum,
        conflicts_with = "graph"
    )]
    #[allow(clippy::option_option)]
    dump_ast: Option<Option<DumpFormat>>,

    /// Dump the AST to stdout with the given format.
    #[arg(long, short, conflicts_with = "graph")]
    trace: bool,

    /// Use vi mode in the REPL
    #[arg(long = "vi")]
    vi_mode: bool,

    /// Report parsing and execution timings.
    #[arg(long)]
    time: bool,

    #[arg(long, short = 'O', group = "optimizer")]
    optimize: bool,

    #[arg(long, requires = "optimizer")]
    optimizer_statistics: bool,

    /// Generate instruction flowgraph. Default is Graphviz.
    #[arg(
        long,
        value_name = "FORMAT",
        ignore_case = true,
        value_enum,
        group = "graph"
    )]
    #[allow(clippy::option_option)]
    flowgraph: Option<Option<FlowgraphFormat>>,

    /// Specifies the direction of the flowgraph. Default is top-top-bottom.
    #[arg(
        long,
        value_name = "FORMAT",
        ignore_case = true,
        value_enum,
        requires = "graph"
    )]
    flowgraph_direction: Option<FlowgraphDirection>,

    /// Inject debugging object `$boa`.
    #[arg(long)]
    debug_object: bool,

    /// Treats the input files as modules.
    #[arg(long, short = 'm', group = "mod")]
    module: bool,

    /// Root path from where the module resolver will try to load the modules.
    #[arg(long, short = 'r', default_value_os_t = PathBuf::from("."), requires = "mod")]
    root: PathBuf,

    /// Execute a JavaScript expression then exit. Files (see above) will be
    /// executed prior to the expression.
    #[arg(long, short = 'e')]
    expression: Option<String>,
}

impl Opt {
    /// Returns whether a dump flag has been used.
    const fn has_dump_flag(&self) -> bool {
        self.dump_ast.is_some()
    }
}

/// The different types of format available for dumping.
#[derive(Debug, Copy, Clone, Default, ValueEnum)]
enum DumpFormat {
    // NOTE: This can easily support other formats just by
    // adding a field to this enum and adding the necessary
    // implementation. Example: Toml, Html, etc.
    //
    // NOTE: The fields of this enum are not doc comments because
    // arg_enum! macro does not support it.
    /// This is the default format that you get from `std::fmt::Debug`.
    #[default]
    Debug,

    /// This is a minified json format.
    Json,

    /// This is a pretty printed json format.
    JsonPretty,
}

/// Represents the format of the instruction flowgraph.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum FlowgraphFormat {
    /// Generates in graphviz format: <https://graphviz.org/>.
    Graphviz,
    /// Generates in mermaid format: <https://mermaid-js.github.io/mermaid/>.
    Mermaid,
}

/// Represents the direction of the instruction flowgraph.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum FlowgraphDirection {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}

struct Timer<'a> {
    name: &'static str,
    start: Instant,
    counters: &'a mut Vec<(&'static str, Duration)>,
}

impl Drop for Timer<'_> {
    fn drop(&mut self) {
        self.counters.push((self.name, self.start.elapsed()));
    }
}

struct Counters {
    counters: Option<Vec<(&'static str, Duration)>>,
}

impl Counters {
    fn new(enabled: bool) -> Self {
        Self {
            counters: enabled.then_some(Vec::new()),
        }
    }

    fn new_timer(&mut self, name: &'static str) -> Option<Timer<'_>> {
        self.counters.as_mut().map(|counters| Timer {
            name,
            start: Instant::now(),
            counters,
        })
    }
}

impl Drop for Counters {
    fn drop(&mut self) {
        let Some(counters) = self.counters.take() else {
            return;
        };
        if counters.is_empty() {
            return;
        }

        let max_width = counters
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(0)
            .max("Total".len())
            + 1; // +1 for the colon

        let mut total = Duration::ZERO;
        eprintln!();
        for (name, elapsed) in &counters {
            eprintln!(
                "{:<width$} {elapsed:.2?}",
                format!("{name}:"),
                width = max_width
            );
            total += *elapsed;
        }
        if counters.len() > 1 {
            eprintln!("{:<width$} {total:.2?}", "Total:", width = max_width);
        }
    }
}

/// Dumps the AST to stdout with format controlled by the given arguments.
///
/// Returns a error of type String with a error message,
/// if the source has a syntax or parsing error.
fn dump<R: ReadChar>(src: Source<'_, R>, args: &Opt, context: &mut Context) -> Result<()> {
    if let Some(arg) = args.dump_ast {
        let mut counters = Counters::new(args.time);
        let arg = arg.unwrap_or_default();
        let mut parser = boa_parser::Parser::new(src);
        let dump =
            if args.module {
                let scope = context.realm().scope().clone();
                let module = {
                    let _timer = counters.new_timer("Parsing");
                    parser
                        .parse_module(&scope, context.interner_mut())
                        .map_err(|e| eyre!("Uncaught SyntaxError: {e}"))?
                };
                let _timer = counters.new_timer("AST generation");
                match arg {
                    DumpFormat::Json => serde_json::to_string(&module)
                        .expect("could not convert AST to a JSON string"),
                    DumpFormat::JsonPretty => serde_json::to_string_pretty(&module)
                        .expect("could not convert AST to a pretty JSON string"),
                    DumpFormat::Debug => format!("{module:#?}"),
                }
            } else {
                let scope = context.realm().scope().clone();
                let mut script = {
                    let _timer = counters.new_timer("Parsing");
                    parser
                        .parse_script(&scope, context.interner_mut())
                        .map_err(|e| eyre!("Uncaught SyntaxError: {e}"))?
                };

                if args.optimize {
                    context.optimize_statement_list(script.statements_mut());
                }

                let _timer = counters.new_timer("AST generation");
                match arg {
                    DumpFormat::Json => serde_json::to_string(&script)
                        .expect("could not convert AST to a JSON string"),
                    DumpFormat::JsonPretty => serde_json::to_string_pretty(&script)
                        .expect("could not convert AST to a pretty JSON string"),
                    DumpFormat::Debug => format!("{script:#?}"),
                }
            };
        drop(counters);
        println!("{dump}");
    }

    Ok(())
}

fn generate_flowgraph<R: ReadChar>(
    context: &mut Context,
    src: Source<'_, R>,
    format: FlowgraphFormat,
    direction: Option<FlowgraphDirection>,
) -> Result<String> {
    let script = Script::parse(src, None, context).map_err(|e| e.into_erased(context))?;
    let code = script
        .codeblock(context)
        .map_err(|e| e.into_erased(context))?;
    let direction = match direction {
        Some(FlowgraphDirection::TopToBottom) | None => Direction::TopToBottom,
        Some(FlowgraphDirection::BottomToTop) => Direction::BottomToTop,
        Some(FlowgraphDirection::LeftToRight) => Direction::LeftToRight,
        Some(FlowgraphDirection::RightToLeft) => Direction::RightToLeft,
    };
    let mut graph = Graph::new(direction);
    code.to_graph(graph.subgraph(String::default()));
    let result = match format {
        FlowgraphFormat::Graphviz => graph.to_graphviz_format(),
        FlowgraphFormat::Mermaid => graph.to_mermaid_format(),
    };
    Ok(result)
}

#[must_use]
fn uncaught_error(error: &JsError) -> String {
    format!("{}: {}\n", "Uncaught".red(), error.to_string().red())
}

#[must_use]
fn uncaught_job_error(error: &JsError) -> String {
    format!(
        "{}: {}\n",
        "Uncaught error (during job evaluation)".red(),
        error.to_string().red()
    )
}

fn evaluate_expr(
    line: &str,
    args: &Opt,
    context: &mut Context,
    printer: &SharedExternalPrinterLogger,
) -> Result<()> {
    if args.has_dump_flag() {
        dump(Source::from_bytes(line), args, context)?;
    } else if let Some(flowgraph) = args.flowgraph {
        match generate_flowgraph(
            context,
            Source::from_bytes(line),
            flowgraph.unwrap_or(FlowgraphFormat::Graphviz),
            args.flowgraph_direction,
        ) {
            Ok(v) => println!("{v}"),
            Err(v) => eprintln!("{v:?}"),
        }
    } else {
        let mut counters = Counters::new(args.time);
        let script = {
            let _timer = counters.new_timer("Parsing");
            Script::parse(Source::from_bytes(line), None, context)
        };

        match script {
            Ok(script) => {
                let result = {
                    let _timer = counters.new_timer("Execution");
                    let result = script.evaluate(context);
                    if let Err(err) = context.run_jobs() {
                        printer.print(uncaught_job_error(&err));
                    }
                    result
                };
                match result {
                    Ok(v) => printer.print(format!("{}\n", v.display())),
                    Err(ref v) => printer.print(uncaught_error(v)),
                }
            }
            Err(ref v) => printer.print(uncaught_error(v)),
        }
    }

    Ok(())
}

fn evaluate_file(
    file: &Path,
    args: &Opt,
    context: &mut Context,
    loader: &SimpleModuleLoader,
    printer: &SharedExternalPrinterLogger,
) -> Result<()> {
    if args.has_dump_flag() {
        return dump(Source::from_filepath(file)?, args, context);
    }

    if let Some(flowgraph) = args.flowgraph {
        let flowgraph = generate_flowgraph(
            context,
            Source::from_filepath(file)?,
            flowgraph.unwrap_or(FlowgraphFormat::Graphviz),
            args.flowgraph_direction,
        )?;

        println!("{flowgraph}");

        return Ok(());
    }

    if args.module {
        let source = Source::from_filepath(file)?;
        let mut counters = Counters::new(args.time);
        let module = {
            let _timer = counters.new_timer("Parsing");
            Module::parse(source, None, context)
        };
        let module = module.map_err(|e| e.into_erased(context))?;

        loader.insert(
            file.canonicalize()
                .wrap_err("could not canonicalize input file path")?,
            module.clone(),
        );

        let promise = {
            let _timer = counters.new_timer("Execution");
            let promise = module.load_link_evaluate(context);
            context.run_jobs().map_err(|err| err.into_erased(context))?;
            Ok::<_, JsErasedError>(promise)
        }?;
        let result = promise.state();

        return match result {
            PromiseState::Pending => Err(eyre!("module didn't execute")),
            PromiseState::Fulfilled(_) => Ok(()),
            PromiseState::Rejected(err) => {
                Err(JsError::from_opaque(err).into_erased(context).into())
            }
        };
    }

    let source = Source::from_filepath(file)?;
    let mut counters = Counters::new(args.time);
    let script = {
        let _timer = counters.new_timer("Parsing");
        Script::parse(source, None, context)
    };
    let script = script.map_err(|e| e.into_erased(context))?;

    let result = {
        let _timer = counters.new_timer("Execution");
        let result = script.evaluate(context);
        context.run_jobs().map_err(|err| err.into_erased(context))?;
        result
    };

    match result {
        Ok(v) => {
            if !v.is_undefined() {
                println!("{}", v.display());
            }
        }
        Err(v) => printer.print(uncaught_error(&v)),
    }

    Ok(())
}

fn evaluate_files(
    args: &Opt,
    context: &mut Context,
    loader: &SimpleModuleLoader,
    printer: &SharedExternalPrinterLogger,
) -> Result<()> {
    for file in &args.files {
        evaluate_file(file, args, context, loader, printer)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_location_section(false)
        .display_env_section(false)
        .install()?;

    #[cfg(feature = "dhat")]
    let _profiler = dhat::Profiler::new_heap();

    let args = Opt::parse();

    // A channel of expressions to run.
    let (sender, receiver) = std::sync::mpsc::channel::<String>();
    let printer = SharedExternalPrinterLogger::new();

    let executor = Rc::new(Executor::new(printer.clone()));
    let loader = Rc::new(SimpleModuleLoader::new(&args.root).map_err(|e| eyre!(e.to_string()))?);
    let mut context = ContextBuilder::new()
        .job_executor(executor)
        .module_loader(loader.clone())
        .build()
        .map_err(|e| eyre!(e.to_string()))?;

    // Strict mode
    context.strict(args.strict);

    // Add `console`.
    add_runtime(printer.clone(), &mut context);

    // Trace Output
    context.set_trace(args.trace);

    if args.debug_object {
        init_boa_debug_object(&mut context);
    }

    // Configure optimizer options
    let mut optimizer_options = OptimizerOptions::empty();
    optimizer_options.set(OptimizerOptions::STATISTICS, args.optimizer_statistics);
    optimizer_options.set(OptimizerOptions::OPTIMIZE_ALL, args.optimize);
    context.set_optimizer_options(optimizer_options);

    if !args.files.is_empty() {
        evaluate_files(&args, &mut context, &loader, &printer)?;

        if let Some(ref expr) = args.expression {
            evaluate_expr(expr, &args, &mut context, &printer)?;
        }

        return Ok(());
    } else if let Some(ref expr) = args.expression {
        evaluate_expr(expr, &args, &mut context, &printer)?;
        return Ok(());
    } else if !io::stdin().is_terminal() {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .wrap_err("failed to read stdin")?;
        return if input.is_empty() {
            Ok(())
        } else {
            evaluate_expr(&input, &args, &mut context, &printer)
        };
    }

    let handle = start_readline_thread(sender, printer.clone(), args.vi_mode);

    loop {
        match receiver.try_recv() {
            Ok(line) => {
                evaluate_expr(&line, &args, &mut context, &printer)?;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => break,
        }

        if let Err(err) = context.run_jobs() {
            printer.print(uncaught_job_error(&err));
        }
        thread::sleep(Duration::from_millis(10));
    }

    handle.join().expect("failed to join thread");

    Ok(())
}

fn readline_thread_main(
    sender: &Sender<String>,
    printer_out: &SharedExternalPrinterLogger,
    vi_mode: bool,
) -> Result<()> {
    let config = Config::builder()
        .keyseq_timeout(Some(1))
        .edit_mode(if vi_mode {
            EditMode::Vi
        } else {
            EditMode::Emacs
        })
        .build();

    let mut editor =
        Editor::with_config(config).wrap_err("failed to set the editor configuration")?;
    if let Ok(printer) = editor.create_external_printer() {
        printer_out.set(printer);
    }

    // Check if the history file exists. If it doesn't, create it.
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(CLI_HISTORY)?;
    editor
        .load_history(CLI_HISTORY)
        .wrap_err("failed to read history file `.boa_history`")?;
    let readline = ">> ";
    editor.set_helper(Some(helper::RLHelper::new(readline)));

    loop {
        match editor.readline(readline) {
            Ok(line) if line == ".exit" => break,
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,

            Ok(line) => {
                let line = line.trim_end();
                editor.add_history_entry(line).map_err(io::Error::other)?;
                sender.send(line.to_string())?;
                thread::sleep(Duration::from_millis(10));
            }

            Err(err) => {
                let final_error = eyre!("could not read the next line of the input");
                let final_error = if let Err(e) = editor.save_history(CLI_HISTORY) {
                    final_error.error(e)
                } else {
                    final_error
                };
                return Err(final_error.error(err));
            }
        }
    }

    editor.save_history(CLI_HISTORY)?;

    Ok(())
}

/// Create the readline thread which sends lines from stdin back to the main thread.
fn start_readline_thread(
    sender: Sender<String>,
    printer_out: SharedExternalPrinterLogger,
    vi_mode: bool,
) -> thread::JoinHandle<()> {
    thread::spawn(
        move || match readline_thread_main(&sender, &printer_out, vi_mode) {
            Ok(()) => {}
            Err(e) => eprintln!("readline thread failed: {e}"),
        },
    )
}

/// Adds the CLI runtime to the context with default options.
fn add_runtime(printer: SharedExternalPrinterLogger, context: &mut Context) {
    boa_runtime::register(
        (
            boa_runtime::extensions::ConsoleExtension(printer),
            #[cfg(feature = "fetch")]
            boa_runtime::extensions::FetchExtension(
                boa_runtime::fetch::BlockingReqwestFetcher::default(),
            ),
        ),
        None,
        context,
    )
    .expect("should not fail while registering the runtime");
}

#[allow(clippy::struct_field_names)]
struct Executor {
    promise_jobs: RefCell<VecDeque<PromiseJob>>,
    async_jobs: RefCell<VecDeque<NativeAsyncJob>>,
    timeout_jobs: RefCell<BTreeMap<JsInstant, Vec<TimeoutJob>>>,
    generic_jobs: RefCell<VecDeque<GenericJob>>,

    printer: SharedExternalPrinterLogger,
}

impl Executor {
    fn new(printer: SharedExternalPrinterLogger) -> Self {
        Self {
            promise_jobs: RefCell::default(),
            async_jobs: RefCell::default(),
            timeout_jobs: RefCell::default(),
            generic_jobs: RefCell::default(),
            printer,
        }
    }

    fn is_empty(&self, context: &mut Context) -> bool {
        let now = context.clock().now();

        self.promise_jobs.borrow().is_empty()
            && self.async_jobs.borrow().is_empty()
            // The timeout jobs queue is empty IF there are no jobs to execute right now.
            && !self.timeout_jobs.borrow().iter().any(|(t, _)| &now > t)
            && self.generic_jobs.borrow().is_empty()
    }

    fn drain_timeout_jobs(&self, context: &mut Context) {
        let now = context.clock().now();

        let mut timeouts_borrow = self.timeout_jobs.borrow_mut();
        let mut jobs_to_keep = timeouts_borrow.split_off(&now);
        jobs_to_keep.retain(|_, jobs| {
            jobs.retain(|job| !job.is_cancelled());
            !jobs.is_empty()
        });
        let jobs_to_run = mem::replace(&mut *timeouts_borrow, jobs_to_keep);
        drop(timeouts_borrow);

        for jobs in jobs_to_run.into_values() {
            for job in jobs {
                if let Err(e) = job.call(context) {
                    self.printer.print(uncaught_job_error(&e));
                }
            }
        }
    }

    fn drain_generic_jobs(&self, context: &mut Context) {
        let job = self.generic_jobs.borrow_mut().pop_front();
        if let Some(generic) = job
            && let Err(err) = generic.call(context)
        {
            self.printer.print(uncaught_job_error(&err));
        }
    }
}

impl JobExecutor for Executor {
    fn enqueue_job(self: Rc<Self>, job: Job, context: &mut Context) {
        match job {
            Job::PromiseJob(job) => self.promise_jobs.borrow_mut().push_back(job),
            Job::AsyncJob(job) => self.async_jobs.borrow_mut().push_back(job),
            Job::TimeoutJob(job) => {
                let now = context.clock().now();
                self.timeout_jobs
                    .borrow_mut()
                    .entry(now + job.timeout())
                    .or_default()
                    .push(job);
            }
            Job::GenericJob(job) => self.generic_jobs.borrow_mut().push_back(job),
            job => self.printer.print(format!("unsupported job type {job:?}")),
        }
    }

    fn run_jobs(self: Rc<Self>, context: &mut Context) -> JsResult<()> {
        future::block_on(self.run_jobs_async(&RefCell::new(context)))
    }

    async fn run_jobs_async(self: Rc<Self>, context: &RefCell<&mut Context>) -> JsResult<()> {
        let mut group = FutureGroup::new();

        loop {
            for job in mem::take(&mut *self.async_jobs.borrow_mut()) {
                group.insert(job.call(context));
            }

            if self.is_empty(&mut context.borrow_mut()) && group.is_empty() {
                return Ok(());
            }

            if let Some(Err(e)) = future::poll_once(group.next()).await.flatten() {
                self.printer.print(uncaught_job_error(&e));
            }

            {
                let context = &mut context.borrow_mut();
                self.drain_timeout_jobs(context);
                self.drain_generic_jobs(context);

                let jobs = mem::take(&mut *self.promise_jobs.borrow_mut());
                for job in jobs {
                    if let Err(e) = job.call(context) {
                        self.printer.print(uncaught_job_error(&e));
                    }
                }
            }
            context.borrow_mut().clear_kept_objects();
            future::yield_now().await;
        }
    }
}
