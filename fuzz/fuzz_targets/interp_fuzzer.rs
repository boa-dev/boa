#![no_main]

use libfuzzer_sys::fuzz_target;

use boa_engine::syntax::Parser;
use boa_engine::Context;
use boa_inputgen::*;

fn do_fuzz(data: FuzzData) -> anyhow::Result<()> {
    let mut context = Context::default();
    let source = data.get_source();
    if let Ok(parsed) = Parser::new(source.as_bytes()).parse_all(&mut context) {
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
