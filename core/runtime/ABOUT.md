# About Boa

Boa is an open-source, experimental ECMAScript Engine written in Rust for
lexing, parsing and executing ECMAScript/JavaScript. Currently, Boa supports some
of the [language][boa-conformance]. More information can be viewed at [Boa's
website][boa-web].

Try out the most recent release with Boa's live demo
[playground][boa-playground].

## Boa Crates

- [**`boa_cli`**][cli] - Boa's CLI && REPL implementation
- [**`boa_ast`**][ast] - Boa's ECMAScript Abstract Syntax Tree.
- [**`boa_engine`**][engine] - Boa's implementation of ECMAScript builtin objects and execution.
- [**`boa_gc`**][gc] - Boa's garbage collector.
- [**`boa_icu_provider`**][icu] - Boa's ICU4X data provider.
- [**`boa_interner`**][interner] - Boa's string interner.
- [**`boa_macros`**][macros] - Boa's macros.
- [**`boa_parser`**][parser] - Boa's lexer and parser.
- [**`boa_profiler`**][profiler] - Boa's code profiler.
- [**`boa_runtime`**][runtime] - Boa's WebAPI features.
- [**`boa_string`**][string] - Boa's ECMAScript string implementation.

[boa-conformance]: https://boajs.dev/conformance
[boa-web]: https://boajs.dev/
[boa-playground]: https://boajs.dev/playground
[ast]: https://docs.rs/boa_ast/latest/boa_ast/index.html
[engine]: https://docs.rs/boa_engine/latest/boa_engine/index.html
[gc]: https://docs.rs/boa_gc/latest/boa_gc/index.html
[interner]: https://docs.rs/boa_interner/latest/boa_interner/index.html
[parser]: https://docs.rs/boa_parser/latest/boa_parser/index.html
[profiler]: https://docs.rs/boa_profiler/latest/boa_profiler/index.html
[icu]: https://docs.rs/boa_icu_provider/latest/boa_icu_provider/index.html
[runtime]: https://docs.rs/boa_runtime/latest/boa_runtime/index.html
[string]: https://docs.rs/boa_string/latest/boa_string/index.html
[macros]: https://docs.rs/boa_macros/latest/boa_macros/index.html
[cli]: https://crates.io/crates/boa_cli
