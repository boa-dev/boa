#![deny(
    unused_qualifications,
    clippy::all,
    unused_qualifications,
    unused_import_braces,
    unused_lifetimes,
    unreachable_pub,
    trivial_numeric_casts,
    rustdoc,
    missing_debug_implementations,
    missing_copy_implementations,
    deprecated_in_future,
    non_ascii_idents,
    rust_2018_compatibility,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style
)]
#![warn(clippy::perf, clippy::single_match_else, clippy::dbg_macro)]
#![allow(
    clippy::missing_inline_in_public_items,
    clippy::cognitive_complexity,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::as_conversions
)]

use boa::{
    exec::Interpreter,
    forward_val,
    realm::Realm,
    syntax::ast::{node::StatementList, token::Token},
};
use colored::*;
use rustyline::{config::Config, error::ReadlineError, EditMode, Editor};
use std::{fs::read_to_string, path::PathBuf};
use structopt::{clap::arg_enum, StructOpt};

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
#[derive(Debug, StructOpt)]
#[structopt(author, about, name = "boa")]
struct Opt {
    /// The JavaScript file(s) to be evaluated.
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,

    /// Dump the token stream to stdout with the given format.
    #[structopt(
        long,
        short = "t",
        value_name = "FORMAT",
        possible_values = &DumpFormat::variants(),
        case_insensitive = true,
        conflicts_with = "dump-ast"
    )]
    dump_tokens: Option<Option<DumpFormat>>,

    /// Dump the ast to stdout with the given format.
    #[structopt(
        long,
        short = "a",
        value_name = "FORMAT",
        possible_values = &DumpFormat::variants(),
        case_insensitive = true
    )]
    dump_ast: Option<Option<DumpFormat>>,

    /// Use vi mode in the REPL
    #[structopt(long = "vi")]
    vi_mode: bool,
}

impl Opt {
    /// Returns whether a dump flag has been used.
    fn has_dump_flag(&self) -> bool {
        self.dump_tokens.is_some() || self.dump_ast.is_some()
    }
}

arg_enum! {
    /// The different types of format available for dumping.
    ///
    // NOTE: This can easily support other formats just by
    // adding a field to this enum and adding the necessary
    // implementation. Example: Toml, Html, etc.
    //
    // NOTE: The fields of this enum are not doc comments because
    // arg_enum! macro does not support it.
    #[derive(Debug)]
    enum DumpFormat {
        // This is the default format that you get from std::fmt::Debug.
        Debug,

        // This is a minified json format.
        Json,

        // This is a pretty printed json format.
        JsonPretty,
    }
}

/// Lexes the given source code into a stream of tokens and return it.
///
/// Returns a error of type String with a message,
/// if the source has a syntax error.
fn lex_source(src: &str) -> Result<Vec<Token>, String> {
    use boa::syntax::lexer::Lexer;

    let mut lexer = Lexer::new(src);
    lexer.lex().map_err(|e| format!("SyntaxError: {}", e))?;
    Ok(lexer.tokens)
}

/// Parses the the token stream into a ast and returns it.
///
/// Returns a error of type String with a message,
/// if the token stream has a parsing error.
fn parse_tokens(tokens: Vec<Token>) -> Result<StatementList, String> {
    use boa::syntax::parser::Parser;

    Parser::new(&tokens)
        .parse_all()
        .map_err(|e| format!("ParsingError: {}", e))
}

/// Dumps the token stream or ast to stdout depending on the given arguments.
///
/// Returns a error of type String with a error message,
/// if the source has a syntax or parsing error.
fn dump(src: &str, args: &Opt) -> Result<(), String> {
    let tokens = lex_source(src)?;

    if let Some(ref arg) = args.dump_tokens {
        match arg {
            Some(format) => match format {
                DumpFormat::Debug => println!("{:#?}", tokens),
                DumpFormat::Json => println!("{}", serde_json::to_string(&tokens).unwrap()),
                DumpFormat::JsonPretty => {
                    println!("{}", serde_json::to_string_pretty(&tokens).unwrap())
                }
            },
            // Default token stream dumping format.
            None => println!("{:#?}", tokens),
        }
    } else if let Some(ref arg) = args.dump_ast {
        let ast = parse_tokens(tokens)?;

        match arg {
            Some(format) => match format {
                DumpFormat::Debug => println!("{:#?}", ast),
                DumpFormat::Json => println!("{}", serde_json::to_string(&ast).unwrap()),
                DumpFormat::JsonPretty => {
                    println!("{}", serde_json::to_string_pretty(&ast).unwrap())
                }
            },
            // Default ast dumping format.
            None => println!("{:#?}", ast),
        }
    }

    Ok(())
}

pub fn main() -> Result<(), std::io::Error> {
    let args = Opt::from_args();

    let realm = Realm::create();

    let mut engine = Interpreter::new(realm);

    for file in &args.files {
        let buffer = read_to_string(file)?;

        if args.has_dump_flag() {
            if let Err(e) = dump(&buffer, &args) {
                eprintln!("{}", e);
            }
        } else {
            match forward_val(&mut engine, &buffer) {
                Ok(v) => print!("{}", v),
                Err(v) => eprint!("{}", v),
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

        let mut editor = Editor::with_config(config);
        let _ = editor.load_history(CLI_HISTORY);
        editor.set_helper(Some(helper::RLHelper::new()));

        let readline = ">> ".color(READLINE_COLOR).bold().to_string();

        loop {
            match editor.readline(&readline) {
                Ok(line) if line == ".exit" => break,
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,

                Ok(line) => {
                    editor.add_history_entry(&line);

                    if args.has_dump_flag() {
                        if let Err(e) = dump(&line, &args) {
                            eprintln!("{}", e);
                        }
                    } else {
                        match forward_val(&mut engine, line.trim_end()) {
                            Ok(v) => println!("{}", v),
                            Err(v) => eprintln!("{}: {}", "Uncaught".red(), v.to_string().red()),
                        }
                    }
                }

                Err(err) => {
                    eprintln!("Unknown error: {:?}", err);
                    break;
                }
            }
        }

        editor.save_history(CLI_HISTORY).unwrap();
    }

    Ok(())
}
