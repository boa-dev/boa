#![deny(unused_qualifications, clippy::correctness, clippy::style)]
#![warn(clippy::perf)]
#![allow(clippy::cognitive_complexity)]

use boa::{exec, exec::Executor, forward_val, realm::Realm};
use std::{fs::read_to_string, path::PathBuf};
use structopt::StructOpt;

/// CLI configuration for Boa.
#[derive(Debug, StructOpt)]
#[structopt(author, about)]
struct Opt {
    /// The javascript file to be evaluated.
    #[structopt(name = "FILE", parse(from_os_str), default_value = "tests/js/test.js")]
    file: PathBuf,
    /// Open a boa shell (WIP).
    #[structopt(short, long)]
    shell: bool,
}

pub fn main() -> Result<(), std::io::Error> {
    let args = Opt::from_args();

    let buffer = read_to_string(args.file)?;

    if args.shell {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);

        match forward_val(&mut engine, &buffer) {
            Ok(v) => print!("{}", v.to_string()),
            Err(v) => eprint!("{}", v.to_string()),
        }
    } else {
        dbg!(exec(&buffer));
    }

    Ok(())
}
