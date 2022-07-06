#![no_main]
#![feature(bench_black_box)]

use std::hint::black_box;

use boa_engine::Context;
use libfuzzer_sys::fuzz_target;

use boa_engine::syntax::Parser;
use boa_inputgen::*;

fn do_fuzz(data: FuzzData) {
    let mut context = Context::default();
    let source = data.get_source();
    drop(black_box(
        Parser::new(source.as_bytes()).parse_all(&mut context),
    ));
}

fuzz_target!(|data: FuzzData| {
    do_fuzz(data);
});
