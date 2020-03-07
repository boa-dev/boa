#![deny(unused_qualifications, clippy::correctness, clippy::style)]
#![warn(clippy::perf)]
#![allow(clippy::cognitive_complexity)]

use boa::builtins::console::log;
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
}
pub fn main() -> Result<(), std::io::Error> {
    let args = Opt::from_args();

    let realm = Realm::create().register_global_func("print", log);

    let mut engine = Executor::new(realm);

    for file in &args.files {
        let buffer = read_to_string(file)?;

        match forward_val(&mut engine, &buffer) {
            Ok(v) => print!("{}", v.to_string()),
            Err(v) => eprint!("{}", v.to_string()),
        }
    }

    if args.files.is_empty() {
        loop {
            let mut buffer = String::new();

            io::stdin().read_line(&mut buffer)?;

            match forward_val(&mut engine, buffer.trim_end()) {
                Ok(v) => println!("{}", v.to_string()),
                Err(v) => eprintln!("{}", v.to_string()),
            }
        }
    }

    Ok(())
}
