#![no_main]

mod common;

use crate::common::FuzzData;
use boa_engine::syntax::Parser;
use boa_interner::ToInternedString;
use libfuzzer_sys::fuzz_target;
use std::error::Error;
use std::io::Cursor;

fn do_fuzz(mut data: FuzzData) -> Result<(), Box<dyn Error>> {
    let original = data.ast.to_interned_string(data.context.interner());

    let mut parser = Parser::new(Cursor::new(&original));

    let before = data.context.interner().len();
    // For a variety of reasons, we may not actually produce valid code here (e.g., nameless function).
    // Fail fast and only make the next checks if we were valid.
    if let Ok(first) = parser.parse_all(&mut data.context) {
        let after_first = data.context.interner().len();
        let first_interned = first.to_interned_string(data.context.interner());

        assert_eq!(
            before,
            after_first,
            "The number of interned symbols changed; a new string was read.\nBefore:\n{:#?}\nAfter:\n{:#?}",
            data.ast,
            first
        );
        let mut parser = Parser::new(Cursor::new(&first_interned));

        // Now, we most assuredly should produce valid code. It has already gone through a first pass.
        let second = parser
            .parse_all(&mut data.context)
            .expect("Could not parse the first-pass interned copy.");
        let after_second = data.context.interner().len();
        assert_eq!(
            after_first,
            after_second,
            "The number of interned symbols changed; a new string was read.\nBefore:\n{:#?}\nAfter:\n{:#?}",
            first,
            second
        );
        assert_eq!(
            first,
            second,
            "Expected the same AST after two intern passes, but found dissimilar.\nFirst:\n{}\nSecond:\n{}",
            first_interned,
            second.to_interned_string(data.context.interner())
        );
    }
    Ok(())
}

fuzz_target!(|data: FuzzData| {
    let _ = do_fuzz(data);
});
