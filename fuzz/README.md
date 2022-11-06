# boa_engine-fuzz

This directory contains fuzzers which can be used to automatically identify faults present in Boa. All the fuzzers in
this directory are [grammar-aware](https://www.fuzzingbook.org/html/Grammars.html) (based on
[Arbitrary](https://docs.rs/arbitrary/latest/arbitrary/)) and coverage-guided. See [common.rs](fuzz/fuzz_targets/common.rs)
for implementation specifics.

You can run any fuzzer you wish with the following command (replacing `your-fuzzer` with a fuzzer availble in
fuzz_targets, e.g. `parser-idempotency`):

```bash
cargo fuzz run -s none your-fuzzer
```

Note that you may wish to use a different sanitizer option (`-s`) according to what kind of issue you're looking for.
Refer to the [cargo-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html) for details on how to select a
sanitizer and other flags.

## Parser Fuzzer

The parser fuzzer, located in [parser-idempotency.rs](fuzz/fuzz_targets/parser-idempotency.rs), identifies
correctness issues in both the parser and the AST-to-source conversion process (e.g., via `to_interned_string`) by
searching for inputs which are not idempotent over parsing and conversion back to source. It does this by doing the
following:

1. Generate an arbitrary AST
2. Convert that AST to source code with `to_interned_string`; we'll call this the "original source"
3. Parse the original source into an AST; we'll call this the "first AST"
   - Arbitrary ASTs aren't guaranteed to be parseable; to avoid errors caused by this, we discard errors here.
4. Convert the first AST to source code with `to_interned_string`; we'll call this the "first source"
5. Parse the first source into an AST; we'll call this the "second AST"
   - Since the original source was parseable, the first source must be parseable; emit any errors parsing produces.
6. Compare the first AST and the second AST. If they are not equal, emit an error.
   - An error here indicates that either the parser or the AST-to-source conversion lost information or added incorrect
     information, as the inputs parsed between the two should be the same.

In this way, this fuzzer can identify correctness issues present in the parser.
