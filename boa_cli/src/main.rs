#![deny(unused_qualifications, clippy::correctness, clippy::style)]
#![warn(clippy::perf)]
#![allow(clippy::cognitive_complexity)]

use boa::builtins::console::log;
use boa::syntax::ast::{expr::Expr, token::Token};
use boa::{exec::Executor, forward_val, realm::Realm};
use std::io;
use std::{fs::read_to_string, path::PathBuf};
use structopt::StructOpt;
/// CLI configuration for Boa.
#[derive(Debug, StructOpt)]
#[structopt(author, about)]
struct Opt {
    /// The JavaScript file(s) to be evaluated.
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,

    /// Dump the token stream to stdout.
    #[structopt(long, short = "-t", conflicts_with = "dump-ast")]
    dump_tokens: bool,

    /// Dump the ast to stdout.
    #[structopt(long, short = "-a")]
    dump_ast: bool,
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
fn parse_tokens(tokens: Vec<Token>) -> Result<Expr, String> {
    use boa::syntax::parser::Parser;

    Parser::new(tokens)
        .parse_all()
        .map_err(|e| format!("ParsingError: {}", e))
}

/// Dumps the token stream or ast to stdout depending on the given arguments.
///
/// Returns a error of type String with a error message,
/// if the source has a syntax or parsing error.
fn dump(src: &str, args: &Opt) -> Result<(), String> {
    let tokens = lex_source(src)?;

    if args.dump_tokens {
        println!("Tokens: {:#?}", tokens);
    } else if args.dump_ast {
        let ast = parse_tokens(tokens)?;
        println!("Ast: {:#?}", ast);
    }

    Ok(())
}

pub fn main() -> Result<(), std::io::Error> {
    let args = Opt::from_args();

    let realm = Realm::create().register_global_func("print", log);

    let mut engine = Executor::new(realm);

    for file in &args.files {
        let buffer = read_to_string(file)?;

        if args.dump_tokens || args.dump_ast {
            match dump(&buffer, &args) {
                Ok(_) => {}
                Err(e) => print!("{}", e),
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

            if args.dump_tokens || args.dump_ast {
                match dump(&buffer, &args) {
                    Ok(_) => {}
                    Err(e) => print!("{}", e),
                }
            } else {
                match forward_val(&mut engine, buffer.trim_end()) {
                    Ok(v) => println!("{}", v.to_string()),
                    Err(v) => eprintln!("{}", v.to_string()),
                }
            }
        }
    }

    Ok(())
}
