#![no_main]

use libfuzzer_sys::fuzz_target;

use boa_engine::syntax::Parser;
use boa_engine::Context;
use boa_inputgen::*;
use boa_interner::Interner;

fn do_fuzz(data: FuzzData) -> anyhow::Result<()> {
    let mut interner = Interner::default();
    let source = data.get_source();
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
