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

use boa::exec;
use std::{env, fs::read_to_string, process::exit};

fn print_usage() {
    println!(
        "Usage:
boa [file.js]
    Interpret and execute file.js
    (if no file given, defaults to tests/js/test.js"
    );
}

pub fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    let read_file;

    match args.len() {
        // No arguments passed, default to "test.js"
        1 => {
            read_file = "tests/js/test.js";
        }
        // One argument passed, assumed this is the test file
        2 => {
            read_file = &args[1];
        }
        // Some other number of arguments passed: not supported
        _ => {
            print_usage();
            exit(1);
        }
    }

    let buffer = read_to_string(read_file)?;
    dbg!(exec(&buffer));
    Ok(())
}
