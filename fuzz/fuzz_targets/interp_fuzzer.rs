#![no_main]

use libfuzzer_sys::fuzz_target;

use boa_engine::syntax::Parser;
use boa_engine::Context;
use boa_inputgen::*;
use boa_interner::{Interner, ToInternedString};

fn do_fuzz(data: FuzzData) -> anyhow::Result<()> {
    let vars = data.vars;
    if vars.is_empty() {
        return Ok(());
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
    if let Ok(parsed) = Parser::new(source.as_bytes(), false).parse_all(&mut interner) {
        let mut context = Context::new(interner);
        context.set_max_insns(1 << 12);
        if let Ok(compiled) = context.compile(&parsed) {
            let _ = context.execute(compiled);
        }
    }
    Ok(())
}

fuzz_target!(|data: FuzzData| {
    let _ = do_fuzz(data);
});
