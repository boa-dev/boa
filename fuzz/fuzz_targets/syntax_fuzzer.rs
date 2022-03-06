#![no_main]
#![feature(bench_black_box)]

use std::hint::black_box;

use libfuzzer_sys::fuzz_target;

use boa_engine::syntax::Parser;
use boa_inputgen::*;
use boa_interner::{Interner, ToInternedString};

fn do_fuzz(data: FuzzData) {
    let vars = data.vars;
    if vars.is_empty() {
        return;
    }
    let mut sample = data.sample;
    let mut interner = Interner::with_capacity(vars.len());
    let syms = vars
        .into_iter()
        .map(|var| interner.get_or_intern(var.name))
        .collect::<Vec<_>>();
    replace_syms(&syms, &mut sample);

    // commit crimes
    let source = sample.to_interned_string(&interner);
    drop(black_box(
        Parser::new(source.as_bytes(), false).parse_all(&mut interner),
    ));
}

fuzz_target!(|data: FuzzData| {
    do_fuzz(data);
});
