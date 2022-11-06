#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![warn(
    clippy::perf,
    clippy::single_match_else,
    clippy::dbg_macro,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::struct_excessive_bools,
    clippy::doc_markdown,
    clippy::semicolon_if_nothing_returned,
    clippy::pedantic
)]
#![deny(
    clippy::all,
    clippy::cast_lossless,
    clippy::redundant_closure_for_method_calls,
    clippy::use_self,
    clippy::unnested_or_patterns,
    clippy::trivially_copy_pass_by_ref,
    clippy::needless_pass_by_value,
    clippy::match_wildcard_for_single_variants,
    clippy::map_unwrap_or,
    unused_qualifications,
    unused_import_braces,
    unused_lifetimes,
    unreachable_pub,
    trivial_numeric_casts,
    // rustdoc,
    missing_debug_implementations,
    missing_copy_implementations,
    deprecated_in_future,
    meta_variable_misuse,
    non_ascii_idents,
    rust_2018_compatibility,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::cast_ptr_alignment,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::unreadable_literal,
    clippy::missing_inline_in_public_items,
    clippy::cognitive_complexity,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::as_conversions,
    clippy::let_unit_value,
    rustdoc::missing_doc_code_examples
)]

use boa_ast::StatementList;
use boa_engine::Context;
use clap::{Parser, ValueEnum, ValueHint};
use colored::{Color, Colorize};
use rustyline::{config::Config, error::ReadlineError, EditMode, Editor};
use std::{fs::read, fs::OpenOptions, io, path::PathBuf};
mod helper;

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[cfg_attr(
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
    global_allocator
)]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

/// CLI configuration for Boa.
static CLI_HISTORY: &str = ".boa_history";

const READLINE_COLOR: Color = Color::Cyan;

// Added #[allow(clippy::option_option)] because to StructOpt an Option<Option<T>>
// is an optional argument that optionally takes a value ([--opt=[val]]).
// https://docs.rs/structopt/0.3.11/structopt/#type-magic
#[allow(clippy::option_option)]
#[derive(Debug, Parser)]
#[command(author, version, about, name = "boa")]
struct Opt {
    /// The JavaScript file(s) to be evaluated.
    #[arg(name = "FILE", value_hint = ValueHint::FilePath)]
    files: Vec<PathBuf>,

    /// Dump the AST to stdout with the given format.
    #[arg(
        long,
        short = 'a',
        value_name = "FORMAT",
        ignore_case = true,
        value_enum
    )]
    dump_ast: Option<Option<DumpFormat>>,

    /// Dump the AST to stdout with the given format.
    #[arg(long, short)]
    trace: bool,

    /// Use vi mode in the REPL
    #[arg(long = "vi")]
    vi_mode: bool,
}

impl Opt {
    /// Returns whether a dump flag has been used.
    fn has_dump_flag(&self) -> bool {
        self.dump_ast.is_some()
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum DumpFormat {
    /// The different types of format available for dumping.
    // NOTE: This can easily support other formats just by
    // adding a field to this enum and adding the necessary
    // implementation. Example: Toml, Html, etc.
    //
    // NOTE: The fields of this enum are not doc comments because
    // arg_enum! macro does not support it.

    // This is the default format that you get from std::fmt::Debug.
    Debug,

    // This is a minified json format.
    Json,

    // This is a pretty printed json format.
    JsonPretty,
}

/// Parses the the token stream into an AST and returns it.
///
/// Returns a error of type String with a message,
/// if the token stream has a parsing error.
fn parse_tokens<S>(src: S, context: &mut Context) -> Result<StatementList, String>
where
    S: AsRef<[u8]>,
{
    let src_bytes = src.as_ref();
    boa_parser::Parser::new(src_bytes)
        .parse_all(context.interner_mut())
        .map_err(|e| format!("ParsingError: {e}"))
}

/// Dumps the AST to stdout with format controlled by the given arguments.
///
/// Returns a error of type String with a error message,
/// if the source has a syntax or parsing error.
fn dump<S>(src: S, args: &Opt, context: &mut Context) -> Result<(), String>
where
    S: AsRef<[u8]>,
{
    if let Some(ref arg) = args.dump_ast {
        let ast = parse_tokens(src, context)?;

        match arg {
            Some(format) => match format {
                DumpFormat::Debug => println!("{ast:#?}"),
                DumpFormat::Json => println!(
                    "{}",
                    serde_json::to_string(&ast).expect("could not convert AST to a JSON string")
                ),
                DumpFormat::JsonPretty => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&ast)
                            .expect("could not convert AST to a pretty JSON string")
                    );
                }
            },
            // Default ast dumping format.
            None => println!("{ast:#?}"),
        }
    }

    Ok(())
}

pub fn main() -> Result<(), io::Error> {
    let args = Opt::parse();

    let mut context = Context::default();

    // Trace Output
    context.set_trace(args.trace);

    for file in &args.files {
        let buffer = read(file)?;

        if args.has_dump_flag() {
            if let Err(e) = dump(&buffer, &args, &mut context) {
                eprintln!("{e}");
            }
        } else {
            match context.eval(&buffer) {
                Ok(v) => println!("{}", v.display()),
                Err(v) => eprintln!("Uncaught {v}"),
            }
        }
    }

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
        editor.set_helper(Some(helper::RLHelper::new()));

        let readline = ">> ".color(READLINE_COLOR).bold().to_string();

        loop {
            match editor.readline(&readline) {
                Ok(line) if line == ".exit" => break,
                Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,

                Ok(line) => {
                    editor.add_history_entry(&line);

                    if args.has_dump_flag() {
                        if let Err(e) = dump(&line, &args, &mut context) {
                            eprintln!("{e}");
                        }
                    } else {
                        match context.eval(line.trim_end()) {
                            Ok(v) => println!("{}", v.display()),
                            Err(v) => {
                                eprintln!("{}: {}", "Uncaught".red(), v.to_string().red());
                            }
                        }
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
    }

    Ok(())
}
