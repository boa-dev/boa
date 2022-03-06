#![no_main]
#![feature(bench_black_box)]

use std::hint::black_box;

use libfuzzer_sys::fuzz_target;

use boa_engine::syntax::Parser;
use boa_inputgen::*;
use boa_interner::Interner;

fn do_fuzz(data: FuzzData) {
    let mut interner = Interner::default();
    let source = data.get_source();
    drop(black_box(
        Parser::new(source.as_bytes(), false).parse_all(&mut interner),
    ));
}

fuzz_target!(|data: FuzzData| {
    do_fuzz(data);
});
