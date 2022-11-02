#![no_main]

mod common;

use crate::common::FuzzData;
use boa_engine::syntax::Parser;
use boa_interner::ToInternedString;
use libfuzzer_sys::fuzz_target;
use std::error::Error;
use std::io::Cursor;

fn do_fuzz(mut data: FuzzData) -> Result<(), Box<dyn Error>> {
    let interned = data.ast.to_interned_string(data.context.interner());

    let mut parser = Parser::new(Cursor::new(&interned));

    let before = data.context.interner().len();
    // For a variety of reasons, we may not actually produce valid code here (e.g., nameless function).
    // Fail fast and only make the next checks if we were valid.
    if let Ok(other) = parser.parse_all(&mut data.context) {
        let after = data.context.interner().len();

        assert_eq!(
            before,
            after,
            "The number of interned symbols changed; a new string was read.\nBefore:\n{}\nAfter:\n{}",
            interned,
            other.to_interned_string(data.context.interner())
        );
        assert_eq!(
            interned,
            other.to_interned_string(data.context.interner()),
            "ASTs before and after did not match; AST after:\n{:#?}",
            other
        );
    }
    Ok(())
}

fuzz_target!(|data: FuzzData| {
    let _ = do_fuzz(data);
});
