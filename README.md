# Boa

<p align="center">
  <a href="https://boajs.dev/">
    <img
      alt="Boa Logo"
      src="./assets/logo.svg"
      width="30%"
    />
    </a>
</p>

This is an experimental Javascript lexer, parser and interpreter written in Rust.
Currently, it has support for some of the language.

[![Build Status][build_badge]][build_link]
[![codecov](https://codecov.io/gh/boa-dev/boa/branch/main/graph/badge.svg)](https://codecov.io/gh/boa-dev/boa)
[![Crates.io](https://img.shields.io/crates/v/boa_engine.svg)](https://crates.io/crates/boa_engine)
[![Docs.rs](https://docs.rs/boa_engine/badge.svg)](https://docs.rs/boa_engine)
[![Discord](https://img.shields.io/discord/595323158140158003?logo=discord)](https://discord.gg/tUFFk9Y)

[build_badge]: https://github.com/boa-dev/boa/actions/workflows/rust.yml/badge.svg?event=push&branch=main
[build_link]: https://github.com/boa-dev/boa/actions/workflows/rust.yml?query=event%3Apush+branch%3Amain

## Live Demo (WASM)

<https://boajs.dev/boa/playground/>

You can get more verbose errors when running from the command line.

## Conformance

To know how much of the _ECMAScript_ specification does Boa cover, you can check out results
running the _ECMASCript Test262_ test suite [here](https://boajs.dev/boa/test262/).

## Contributing

Please, check the [CONTRIBUTING.md](CONTRIBUTING.md) file to know how to
contribute in the project. You will need Rust installed and an editor. We have
some configurations ready for VSCode.

### Debugging

Check [debugging.md](./docs/debugging.md) for more info on debugging.

### Web Assembly

This interpreter can be exposed to JavaScript!
You can build the example locally with:

```shell
npm run build
```

In the console you can use `window.evaluate` to pass JavaScript in.
To develop on the web assembly side you can run:

```shell
npm run serve
```

then go to `http://localhost:8080`.

## Usage

- Clone this repo.
- Run `cargo build`
- Run with `cargo run -- test.js` where `test.js` is an existing JS file with any JS valid code. The file has to be in the root directory.
- If any JS doesn't work then it's a bug. Please raise an [issue](https://github.com/boa-dev/boa/issues/)!

### Example

![Example](docs/img/latestDemo.gif)

## Command-line Options

```
Usage: boa [OPTIONS] [FILE]...

Arguments:
  [FILE]...  The JavaScript file(s) to be evaluated

Options:
      --strict                        Run in strict mode
  -a, --dump-ast [<FORMAT>]           Dump the AST to stdout with the given format [possible values: debug, json, json-pretty]
  -t, --trace                         Dump the AST to stdout with the given format
      --vi                            Use vi mode in the REPL
  -O, --optimize
      --optimizer-statistics
      --flowgraph [<FORMAT>]          Generate instruction flowgraph. Default is Graphviz [possible values: graphviz, mermaid]
      --flowgraph-direction <FORMAT>  Specifies the direction of the flowgraph. Default is top-top-bottom [possible values: top-to-bottom, bottom-to-top, left-to-right, right-to-left]
      --debug-object                  Inject debugging object `$boa`
  -h, --help                          Print help (see more with '--help')
  -V, --version                       Print version
```

## Roadmap

See [Milestones](https://github.com/boa-dev/boa/milestones).

## Benchmarks

See [Benchmarks](https://boajs.dev/boa/dev/bench/).

## Profiling

See [Profiling](./docs/profiling.md).

## Changelog

See [CHANGELOG.md](./CHANGELOG.md).

## Communication

Feel free to contact us on [Discord](https://discord.gg/tUFFk9Y).

## License

This project is licensed under the [Unlicense](./LICENSE-UNLICENSE) or [MIT](./LICENSE-MIT) licenses, at your option.
