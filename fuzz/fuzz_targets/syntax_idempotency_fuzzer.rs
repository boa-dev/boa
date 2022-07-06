#![no_main]

use libfuzzer_sys::fuzz_target;

use boa_engine::syntax::Parser;
use boa_engine::Context;
use boa_inputgen::*;
use boa_interner::ToInternedString;

fn do_fuzz(data: FuzzData) {
    let mut context = Context::default();
    let source = data.get_source();
    if let Ok(parsed) = Parser::new(source.as_bytes()).parse_all(&mut context) {
        let mut alt_ctx = Context::default();
        let new_source = parsed.to_interned_string(context.interner());
        match Parser::new(new_source.as_bytes()).parse_all(&mut alt_ctx) {
            Ok(alternate) => {
                assert_eq!(
                    parsed, alternate,
                    "Expected `{}` ({:?}) but found `{}` ({:?})",
                    source, parsed, new_source, alternate
                );
            }
            Err(_) => {
                panic!(
                    "Expected `{}` ({:?}), but couldn't parse the resulting interned string `{}`",
                    source, parsed, new_source
                );
            }
        }
    }
}

fuzz_target!(|data: FuzzData| {
    do_fuzz(data);
});
