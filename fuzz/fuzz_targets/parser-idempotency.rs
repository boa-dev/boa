#![no_main]

mod common;

use crate::common::FuzzData;
use boa_interner::ToInternedString;
use boa_parser::Parser;
use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::Corpus;
use std::error::Error;
use std::io::Cursor;

/// Fuzzer test harness. This function accepts the arbitrary AST and performs the fuzzing operation.
///
/// See [README.md](../README.md) for details on the design of this fuzzer.
fn do_fuzz(mut data: FuzzData) -> Result<(), Box<dyn Error>> {
    let original = data.ast.to_interned_string(&data.interner);

    let mut parser = Parser::new(Cursor::new(&original));

    let before = data.interner.len();
    // For a variety of reasons, we may not actually produce valid code here (e.g., nameless function).
    // Fail fast and only make the next checks if we were valid.
    if let Ok(first) = parser.parse_all(&mut data.interner) {
        let after_first = data.interner.len();
        let first_interned = first.to_interned_string(&data.interner);

        assert_eq!(
            before,
            after_first,
            "The number of interned symbols changed; a new string was read.\nBefore:\n{}\nAfter:\n{}\nBefore (AST):\n{:#?}\nAfter (AST):\n{:#?}",
            original,
            first_interned,
            data.ast,
            first
        );
        let mut parser = Parser::new(Cursor::new(&first_interned));

        // Now, we most assuredly should produce valid code. It has already gone through a first pass.
        let second = parser
            .parse_all(&mut data.interner)
            .expect("Could not parse the first-pass interned copy.");
        let second_interned = second.to_interned_string(&data.interner);
        let after_second = data.interner.len();
        assert_eq!(
            after_first,
            after_second,
            "The number of interned symbols changed; a new string was read.\nBefore:\n{}\nAfter:\n{}\nBefore (AST):\n{:#?}\nAfter (AST):\n{:#?}",
            first_interned,
            second_interned,
            first,
            second
        );
        assert_eq!(
            first,
            second,
            "Expected the same AST after two intern passes, but found dissimilar.\nOriginal:\n{}\nFirst:\n{}\nSecond:\n{}",
            original,
            first_interned,
            second_interned,
        );
    }
    Ok(())
}

// Fuzz harness wrapper to expose it to libfuzzer (and thus cargo-fuzz)
// See: https://rust-fuzz.github.io/book/cargo-fuzz.html
fuzz_target!(|data: FuzzData| -> Corpus {
    if do_fuzz(data).is_ok() {
        Corpus::Keep
    } else {
        Corpus::Reject
    }
});
