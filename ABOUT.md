# About Boa

Boa is an open-source, experimental ECMAScript Engine written in Rust for
lexing, parsing and executing ECMAScript/JavaScript Currently, Boa supports some
of the [language][boa-conformance]. More information can be viewed at [Boa's
website][boa-web].

Try out the most recent release with Boa's live demo
[playground][boa-playground].

# Boa Crates

- [**`boa_ast`**][ast] - Boa's ECMAScript Abstract Syntax Tree.
- [**`boa_engine`**][engine] - Boa's implementation of ECMAScript builtin objects and
  execution.
- [**`boa_gc`**][gc] - Boa's garbage collector.
- [**`boa_interner`**][interner] - Boa's string interner.
- [**`boa_parser`**][parser] - Boa's lexer and parser.
- [**`boa_profiler`**][profiler] - Boa's code profiler.
- [**`boa_icu_provider`**][icu] - Boa's ICU4X data provider.
- [**`boa_runtime`**][runtime] - Boa's WebAPI features.

[boa-conformance]: https://boajs.dev/boa/test262/
[boa-web]: https://boajs.dev/
[boa-playground]: https://boajs.dev/boa/playground/
[ast]: https://boajs.dev/boa/doc/boa_ast/index.html
[engine]: https://boajs.dev/boa/doc/boa_engine/index.html
[gc]: https://boajs.dev/boa/doc/boa_gc/index.html
[interner]: https://boajs.dev/boa/doc/boa_interner/index.html
[parser]: https://boajs.dev/boa/doc/boa_parser/index.html
[profiler]: https://boajs.dev/boa/doc/boa_profiler/index.html
[icu]: https://boajs.dev/boa/doc/boa_icu_provider/index.html
[runtime]: https://boajs.dev/boa/doc/boa_runtime/index.html