#![deny(unused_qualifications, clippy::correctness, clippy::style)]
#![warn(clippy::perf)]
#![allow(clippy::cognitive_complexity)]

use boa::builtins::console::log;
use boa::serde_json;
use boa::syntax::ast::{node::Node, token::Token};
use boa::{exec::Executor, forward_val, realm::Realm};
use std::io::{self, Write};
use std::{fs::read_to_string, path::PathBuf};
use structopt::clap::arg_enum;
use structopt::StructOpt;

/// CLI configuration for Boa.
//
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
fn parse_tokens(tokens: Vec<Token>) -> Result<Node, String> {
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

    let realm = Realm::create().register_global_func("print", log);

    let mut engine = Executor::new(realm);

    for file in &args.files {
        let buffer = read_to_string(file)?;

        if args.has_dump_flag() {
            match dump(&buffer, &args) {
                Ok(_) => {}
                Err(e) => eprintln!("{}", e),
            }
        } else {
            match forward_val(&mut engine, &buffer) {
                Ok(v) => print!("{}", v.to_string()),
                Err(v) => eprint!("{}", v.to_string()),
            }
        }
    }

    if args.files.is_empty() {
        loop {
            let mut buffer = String::new();

            io::stdin().read_line(&mut buffer)?;

            if args.has_dump_flag() {
                match dump(&buffer, &args) {
                    Ok(_) => {}
                    Err(e) => eprintln!("{}", e),
                }
            } else {
                match forward_val(&mut engine, buffer.trim_end()) {
                    Ok(v) => println!("{}", v.to_string()),
                    Err(v) => eprintln!("{}", v.to_string()),
                }
            }

            // The flush is needed because where in a REPL and we do not want buffering.
            std::io::stdout().flush().unwrap();
        }
    }

    Ok(())
}
