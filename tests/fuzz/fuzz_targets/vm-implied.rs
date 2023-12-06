#![no_main]

mod common;

use crate::common::FuzzSource;
use boa_engine::{Context, JsResult, JsValue};
use boa_parser::Source;
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fn do_fuzz(original: FuzzSource) -> JsResult<JsValue> {
    let mut ctx = Context::builder()
        .interner(original.interner)
        .instructions_remaining(1 << 16)
        .build()
        .unwrap();
    ctx.eval(Source::from_reader(Cursor::new(&original.source), None))
}

fuzz_target!(|original: FuzzSource| {
    let _ = do_fuzz(original);
});
