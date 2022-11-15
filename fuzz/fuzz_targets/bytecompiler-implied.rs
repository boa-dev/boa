#![no_main]

mod common;

use crate::common::FuzzSource;
use boa_engine::Context;
use boa_parser::Parser;
use libfuzzer_sys::{fuzz_target, Corpus};
use std::io::Cursor;

fn do_fuzz(original: FuzzSource) -> Corpus {
    let mut ctx = Context::builder()
        .interner(original.interner)
        .instructions_remaining(0)
        .build();
    let mut parser = Parser::new(Cursor::new(&original.source));
    if let Ok(parsed) = parser.parse_all(ctx.interner_mut()) {
        let _ = ctx.compile(&parsed);
        Corpus::Keep
    } else {
        Corpus::Reject
    }
}

fuzz_target!(|original: FuzzSource| -> Corpus { do_fuzz(original) });
