#![no_main]

mod common;

use crate::common::FuzzSource;
use boa_engine::{Context, Script};
use boa_parser::Source;
use libfuzzer_sys::{fuzz_target, Corpus};
use std::io::Cursor;

fn do_fuzz(original: FuzzSource) -> Corpus {
    let mut ctx = Context::builder()
        .interner(original.interner)
        .instructions_remaining(0)
        .build()
        .unwrap();
    if let Ok(parsed) = Script::parse(
        Source::from_reader(Cursor::new(&original.source), None),
        None,
        &mut ctx,
    ) {
        let _ = parsed.codeblock(&mut ctx);
        Corpus::Keep
    } else {
        Corpus::Reject
    }
}

fuzz_target!(|original: FuzzSource| -> Corpus { do_fuzz(original) });
