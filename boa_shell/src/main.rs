#![deny(unused_qualifications, clippy::correctness, clippy::style)]
#![warn(clippy::perf)]
#![allow(clippy::cognitive_complexity)]

use boa::realm::Realm;
use boa::{exec::Executor, forward_val};
use std::{fs::read_to_string, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str), help = "the javascript file to be evaluated")]
    file: PathBuf,
}

pub fn main() -> Result<(), std::io::Error> {
    let args = Opt::from_args();

    let buffer = read_to_string(args.file)?;
    let realm = Realm::create();
    let mut engine = Executor::new(realm);

    match forward_val(&mut engine, &buffer) {
        Ok(v) => print!("{}", v.to_string()),
        Err(v) => eprint!("{}", v.to_string()),
    }

    Ok(())
}
