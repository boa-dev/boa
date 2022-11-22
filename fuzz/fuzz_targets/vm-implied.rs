#![no_main]

mod common;

use crate::common::FuzzSource;
use boa_engine::{Context, JsResult, JsValue};
use libfuzzer_sys::fuzz_target;

fn do_fuzz(original: FuzzSource) -> JsResult<JsValue> {
    let mut ctx = Context::builder()
        .interner(original.interner)
        .instructions_remaining(1 << 16)
        .build();
    ctx.eval(&original.source)
}

fuzz_target!(|original: FuzzSource| {
    let _ = do_fuzz(original);
});
