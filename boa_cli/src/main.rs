//! A ECMAScript REPL implementation based on boa_engine.
#![doc = include_str!("../ABOUT.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![warn(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    warnings,
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,

    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    missing_docs,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,

    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,

    // clippy allowed by default
    clippy::dbg_macro,

    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
)]
#![allow(clippy::option_if_let_else, clippy::redundant_pub_crate)]

use boa_ast as _;

mod debug;
mod helper;

use boa_engine::{
    builtins::promise::PromiseState,
    context::ContextBuilder,
    job::{FutureJob, JobQueue, NativeJob},
    module::{Module, ModuleLoader, SimpleModuleLoader},
    optimizer::OptimizerOptions,
    property::Attribute,
    script::Script,
    vm::flowgraph::{Direction, Graph},
    Context, JsError, JsNativeError, JsResult, Source,
};
use boa_runtime::Console;
use clap::{Parser, ValueEnum, ValueHint};
use colored::Colorize;
use debug::init_boa_debug_object;
use rustyline::{config::Config, error::ReadlineError, EditMode, Editor};
use std::{
    cell::RefCell, collections::VecDeque, eprintln, fs::read, fs::OpenOptions, io, path::PathBuf,
    println,
};

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[cfg_attr(
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
    global_allocator
)]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

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

#[derive(Debug, Copy, Clone, Default, ValueEnum)]
enum DumpFormat {
    /// The different types of format available for dumping.
    // NOTE: This can easily support other formats just by
    // adding a field to this enum and adding the necessary
    // implementation. Example: Toml, Html, etc.
    //
    // NOTE: The fields of this enum are not doc comments because
    // arg_enum! macro does not support it.

    // This is the default format that you get from std::fmt::Debug.
    #[default]
    Debug,

    // This is a minified json format.
    Json,

    // This is a pretty printed json format.
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
fn dump<S>(src: &S, args: &Opt, context: &mut Context<'_>) -> Result<(), String>
where
    S: AsRef<[u8]> + ?Sized,
{
    if let Some(arg) = args.dump_ast {
        let arg = arg.unwrap_or_default();
        let mut parser = boa_parser::Parser::new(Source::from_bytes(src));
        let dump =
            if args.module {
                let module = parser
                    .parse_module(context.interner_mut())
                    .map_err(|e| format!("Uncaught SyntaxError: {e}"))?;

                match arg {
                    DumpFormat::Json => serde_json::to_string(&module)
                        .expect("could not convert AST to a JSON string"),
                    DumpFormat::JsonPretty => serde_json::to_string_pretty(&module)
                        .expect("could not convert AST to a pretty JSON string"),
                    DumpFormat::Debug => format!("{module:#?}"),
                }
            } else {
                let mut script = parser
                    .parse_script(context.interner_mut())
                    .map_err(|e| format!("Uncaught SyntaxError: {e}"))?;

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

fn generate_flowgraph(
    context: &mut Context<'_>,
    src: &[u8],
    format: FlowgraphFormat,
    direction: Option<FlowgraphDirection>,
) -> JsResult<String> {
    let script = Script::parse(Source::from_bytes(src), None, context)?;
    let code = script.codeblock(context)?;

    let direction = match direction {
        Some(FlowgraphDirection::TopToBottom) | None => Direction::TopToBottom,
        Some(FlowgraphDirection::BottomToTop) => Direction::BottomToTop,
        Some(FlowgraphDirection::LeftToRight) => Direction::LeftToRight,
        Some(FlowgraphDirection::RightToLeft) => Direction::RightToLeft,
    };

    let mut graph = Graph::new(direction);
    code.to_graph(context.interner(), graph.subgraph(String::default()));
    let result = match format {
        FlowgraphFormat::Graphviz => graph.to_graphviz_format(),
        FlowgraphFormat::Mermaid => graph.to_mermaid_format(),
    };
    Ok(result)
}

fn evaluate_files(
    args: &Opt,
    context: &mut Context<'_>,
    loader: &SimpleModuleLoader,
) -> Result<(), io::Error> {
    for file in &args.files {
        let buffer = read(file)?;

        if args.has_dump_flag() {
            if let Err(e) = dump(&buffer, args, context) {
                eprintln!("{e}");
            }
        } else if let Some(flowgraph) = args.flowgraph {
            match generate_flowgraph(
                context,
                &buffer,
                flowgraph.unwrap_or(FlowgraphFormat::Graphviz),
                args.flowgraph_direction,
            ) {
                Ok(v) => println!("{v}"),
                Err(v) => eprintln!("Uncaught {v}"),
            }
        } else if args.module {
            let result = (|| {
                let module = Module::parse(Source::from_bytes(&buffer), None, context)?;

                loader.insert(
                    file.canonicalize()
                        .map_err(|e| JsNativeError::typ().with_message(e.to_string()))?,
                    module.clone(),
                );

                let promise = module.load_link_evaluate(context)?;

                context.run_jobs();
                promise.state()
            })();

            match result {
                Ok(PromiseState::Pending) => {
                    eprintln!("module `{}` didn't execute", file.display());
                }
                Ok(PromiseState::Fulfilled(_)) => {}
                Ok(PromiseState::Rejected(err)) => {
                    eprintln!("Uncaught {}", err.display());

                    if let Ok(err) = JsError::from_opaque(err).try_native(context) {
                        if let Some(cause) = err.cause() {
                            eprintln!("\tCaused by: {cause}");
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Uncaught {err}");

                    if let Ok(err) = err.try_native(context) {
                        if let Some(cause) = err.cause() {
                            eprintln!("\tCaused by: {cause}");
                        }
                    }
                }
            }
        } else {
            match context.eval(Source::from_bytes(&buffer)) {
                Ok(v) => println!("{}", v.display()),
                Err(v) => eprintln!("Uncaught {v}"),
            }
            context.run_jobs();
        }
    }

    Ok(())
}

fn main() -> Result<(), io::Error> {
    let args = Opt::parse();

    let queue: &dyn JobQueue = &Jobs::default();
    let loader = &SimpleModuleLoader::new(&args.root)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let dyn_loader: &dyn ModuleLoader = loader;
    let mut context = ContextBuilder::new()
        .job_queue(queue)
        .module_loader(dyn_loader)
        .build()
        .expect("cannot fail with default global object");

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

    if args.files.is_empty() {
        let config = Config::builder()
            .keyseq_timeout(1)
            .edit_mode(if args.vi_mode {
                EditMode::Vi
            } else {
                EditMode::Emacs
            })
            .build();

        let mut editor =
            Editor::with_config(config).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        // Check if the history file exists. If it does, create it.
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(CLI_HISTORY)?;
        editor.load_history(CLI_HISTORY).map_err(|err| match err {
            ReadlineError::Io(e) => e,
            e => io::Error::new(io::ErrorKind::Other, e),
        })?;
        let readline = ">> ";
        editor.set_helper(Some(helper::RLHelper::new(readline)));

        loop {
            match editor.readline(readline) {
                Ok(line) if line == ".exit" => break,
                Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,

                Ok(line) => {
                    editor
                        .add_history_entry(&line)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                    if args.has_dump_flag() {
                        if let Err(e) = dump(&line, &args, &mut context) {
                            eprintln!("{e}");
                        }
                    } else if let Some(flowgraph) = args.flowgraph {
                        match generate_flowgraph(
                            &mut context,
                            line.trim_end().as_bytes(),
                            flowgraph.unwrap_or(FlowgraphFormat::Graphviz),
                            args.flowgraph_direction,
                        ) {
                            Ok(v) => println!("{v}"),
                            Err(v) => eprintln!("Uncaught {v}"),
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
                        context.run_jobs();
                    }
                }

                Err(err) => {
                    eprintln!("Unknown error: {err:?}");
                    break;
                }
            }
        }

        editor
            .save_history(CLI_HISTORY)
            .expect("could not save CLI history");
    } else {
        evaluate_files(&args, &mut context, loader)?;
    }

    Ok(())
}

/// Adds the CLI runtime to the context.
fn add_runtime(context: &mut Context<'_>) {
    let console = Console::init(context);
    context
        .register_global_property(Console::NAME, console, Attribute::all())
        .expect("the console object shouldn't exist");
}

#[derive(Default)]
struct Jobs(RefCell<VecDeque<NativeJob>>);

impl JobQueue for Jobs {
    fn enqueue_promise_job(&self, job: NativeJob, _: &mut Context<'_>) {
        self.0.borrow_mut().push_back(job);
    }

    fn run_jobs(&self, context: &mut Context<'_>) {
        loop {
            let jobs = std::mem::take(&mut *self.0.borrow_mut());
            if jobs.is_empty() {
                return;
            }
            for job in jobs {
                if let Err(e) = job.call(context) {
                    eprintln!("Uncaught {e}");
                }
            }
        }
    }

    fn enqueue_future_job(&self, future: FutureJob, _: &mut Context<'_>) {
        let job = pollster::block_on(future);
        self.0.borrow_mut().push_back(job);
    }
}
