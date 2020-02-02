#![deny(unused_qualifications, clippy::correctness, clippy::style)]
#![warn(clippy::perf)]
#![allow(clippy::cognitive_complexity)]

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
