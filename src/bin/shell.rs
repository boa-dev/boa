#![forbid(
    warnings,
    anonymous_parameters,
    unused_extern_crates,
    unused_import_braces,
    missing_copy_implementations,
    //trivial_casts,
    variant_size_differences,
    missing_debug_implementations,
    trivial_numeric_casts
)]
// Debug trait derivation will show an error if forbidden.
#![deny(unused_qualifications, unsafe_code)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(
    missing_docs,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::excessive_precision,
    clippy::module_name_repetitions
)]

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

    let mut engine = Executor::new();

    match forward_val(&mut engine, &buffer) {
        Ok(v) => print!("{}", v.to_string()),
        Err(v) => eprint!("{}", v.to_string()),
    }

    Ok(())
}
