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
use rustyline::{EditMode, Editor, config::Config, error::ReadlineError};
use std::{
    cell::RefCell,
    collections::VecDeque,
    eprintln,
    fs::OpenOptions,
    io,
    path::{Path, PathBuf},
    println,
    rc::Rc,
};

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

/// Dumps the AST to stdout with format controlled by the given arguments.
///
/// Returns a error of type String with a error message,
/// if the source has a syntax or parsing error.
fn dump<R: ReadChar>(src: Source<'_, R>, args: &Opt, context: &mut Context) -> Result<()> {
    if let Some(arg) = args.dump_ast {
        let arg = arg.unwrap_or_default();
        let mut parser = boa_parser::Parser::new(src);
        let dump =
            if args.module {
                let scope = context.realm().scope().clone();
                let module = parser
                    .parse_module(&scope, context.interner_mut())
                    .map_err(|e| eyre!("Uncaught SyntaxError: {e}"))?;

                match arg {
                    DumpFormat::Json => serde_json::to_string(&module)
                        .expect("could not convert AST to a JSON string"),
                    DumpFormat::JsonPretty => serde_json::to_string_pretty(&module)
                        .expect("could not convert AST to a pretty JSON string"),
                    DumpFormat::Debug => format!("{module:#?}"),
                }
            } else {
                let scope = context.realm().scope().clone();
                let mut script = parser
                    .parse_script(&scope, context.interner_mut())
                    .map_err(|e| eyre!("Uncaught SyntaxError: {e}"))?;

                if args.optimize {
                    context.optimize_statement_list(script.statements_mut());
                }

                match arg {
                    DumpFormat::Json => serde_json::to_string(&script)
                        .expect("could not convert AST to a JSON string"),
                    DumpFormat::JsonPretty => serde_json::to_string_pretty(&script)
                        .expect("could not convert AST to a pretty JSON string"),
                    DumpFormat::Debug => format!("{script:#?}"),
                }
            };

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

fn evaluate_file(
    file: &Path,
    args: &Opt,
    context: &mut Context,
    loader: &SimpleModuleLoader,
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
        let module = Module::parse(Source::from_filepath(file)?, None, context)
            .map_err(|e| e.into_erased(context))?;

        loader.insert(
            file.canonicalize()
                .wrap_err("could not canonicalize input file path")?,
            module.clone(),
        );

        let promise = module.load_link_evaluate(context);
        context.run_jobs().map_err(|err| err.into_erased(context))?;
        let result = promise.state();

        return match result {
            PromiseState::Pending => Err(eyre!("module didn't execute")),
            PromiseState::Fulfilled(_) => Ok(()),
            PromiseState::Rejected(err) => {
                return Err(JsError::from_opaque(err).into_erased(context).into());
            }
        };
    }

    match context.eval(Source::from_filepath(file)?) {
        Ok(v) => println!("{}", v.display()),
        Err(v) => eprintln!("Uncaught {v}"),
    }
    context
        .run_jobs()
        .map_err(|err| err.into_erased(context).into())
}

fn evaluate_files(args: &Opt, context: &mut Context, loader: &SimpleModuleLoader) {
    for file in &args.files {
        let Err(err) = evaluate_file(file, args, context, loader)
            .wrap_err_with(|| eyre!("could not evaluate file `{}`", file.display()))
        else {
            continue;
        };

        eprintln!("{err:?}");
    }
}

fn main() -> Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_location_section(false)
        .display_env_section(false)
        .install()?;

    #[cfg(feature = "dhat")]
    let _profiler = dhat::Profiler::new_heap();

    let args = Opt::parse();

    let executor = Rc::new(Executor::default());
    let loader = Rc::new(SimpleModuleLoader::new(&args.root).map_err(|e| eyre!(e.to_string()))?);
    let mut context = ContextBuilder::new()
        .job_executor(executor)
        .module_loader(loader.clone())
        .build()
        .map_err(|e| eyre!(e.to_string()))?;

    // Strict mode
    context.strict(args.strict);

    // Add `console`.
    add_runtime(&mut context);

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
        evaluate_files(&args, &mut context, &loader);
        return Ok(());
    }

    let config = Config::builder()
        .keyseq_timeout(Some(1))
        .edit_mode(if args.vi_mode {
            EditMode::Vi
        } else {
            EditMode::Emacs
        })
        .build();

    let mut editor =
        Editor::with_config(config).wrap_err("failed to set the editor configuration")?;
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
                editor.add_history_entry(&line).map_err(io::Error::other)?;

                if args.has_dump_flag()
                    && let Err(e) = dump(Source::from_bytes(&line), &args, &mut context)
                {
                    eprintln!("{e:?}");
                } else if let Some(flowgraph) = args.flowgraph {
                    match generate_flowgraph(
                        &mut context,
                        Source::from_bytes(line.trim_end()),
                        flowgraph.unwrap_or(FlowgraphFormat::Graphviz),
                        args.flowgraph_direction,
                    ) {
                        Ok(v) => println!("{v}"),
                        Err(v) => eprintln!("{v:?}"),
                    }
                } else {
                    match context.eval(Source::from_bytes(line.trim_end())) {
                        Ok(v) => {
                            println!("{}", v.display());
                        }
                        Err(v) => {
                            eprintln!("{}: {}", "Uncaught".red(), v.to_string().red());
                        }
                    }
                    if let Err(err) = context.run_jobs() {
                        eprintln!("{err}");
                    }
                }
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

/// Adds the CLI runtime to the context with default options.
fn add_runtime(context: &mut Context) {
    boa_runtime::register(
        context,
        boa_runtime::RegisterOptions::default()
            .with_fetcher(boa_runtime::fetch::fetchers::ReqwestFetcher::default()),
    )
    .expect("should not fail while registering the runtime");
}

#[derive(Default)]
struct Executor {
    promise_jobs: RefCell<VecDeque<PromiseJob>>,
    async_jobs: RefCell<VecDeque<NativeAsyncJob>>,
}

impl JobExecutor for Executor {
    fn enqueue_job(&self, job: Job, _: &mut Context) {
        match job {
            Job::PromiseJob(job) => self.promise_jobs.borrow_mut().push_back(job),
            Job::AsyncJob(job) => self.async_jobs.borrow_mut().push_back(job),
            job => eprintln!("unsupported job type {job:?}"),
        }
    }

    fn run_jobs(&self, context: &mut Context) -> JsResult<()> {
        loop {
            if self.promise_jobs.borrow().is_empty() && self.async_jobs.borrow().is_empty() {
                return Ok(());
            }

            let jobs = std::mem::take(&mut *self.promise_jobs.borrow_mut());
            for job in jobs {
                if let Err(e) = job.call(context) {
                    eprintln!("Uncaught {e}");
                }
            }

            let async_jobs = std::mem::take(&mut *self.async_jobs.borrow_mut());
            for async_job in async_jobs {
                if let Err(err) = pollster::block_on(async_job.call(&RefCell::new(context))) {
                    eprintln!("Uncaught {err}");
                }
                let jobs = std::mem::take(&mut *self.promise_jobs.borrow_mut());
                for job in jobs {
                    if let Err(e) = job.call(context) {
                        eprintln!("Uncaught {e}");
                    }
                }
            }
        }
    }
}
