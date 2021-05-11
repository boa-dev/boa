# Boa

<p align="center">
    <img
      alt="logo"
      src="./assets/logo.svg"
      width="30%"
    />
</p>

This is an experimental Javascript lexer, parser and interpreter written in Rust.
Currently, it has support for some of the language.

[![Build Status][build_badge]][build_link]
[![codecov](https://codecov.io/gh/boa-dev/boa/branch/master/graph/badge.svg)](https://codecov.io/gh/boa-dev/boa)
[![](http://meritbadge.herokuapp.com/boa)](https://crates.io/crates/boa)
[![](https://docs.rs/Boa/badge.svg)](https://docs.rs/Boa/)
![Discord](https://img.shields.io/discord/595323158140158003?logo=discord)

[build_badge]: https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2Fboa-dev%2Fboa%2Fbadge&style=flat
[build_link]: https://actions-badge.atrox.dev/boa-dev/boa/goto

## Live Demo (WASM)

<https://boa-dev.github.io/boa/>

You can get more verbose errors when running from the command line.

## Development documentation

You can check the internal development docs at <https://boa-dev.github.io/boa/doc>.

## Conformance

To know how much of the ECMAScript specification does Boa cover, you can check out results running the ECMASCript Test262 test suite [here](https://boa-dev.github.io/boa/test262/).

## Benchmarks

See [Benchmarks](https://boa-dev.github.io/boa/dev/bench/).

## Contributing

Please, check the [CONTRIBUTING.md](CONTRIBUTING.md) file to know how to
contribute in the project. You will need Rust installed and an editor. We have
some configurations ready for VSCode.

### Debugging

Check [debugging.md](./docs/debugging.md) for more info on debugging.

### Web Assembly

This interpreter can be exposed to javascript!
You can build the example locally with:

```
$ yarn install
$ yarn serve
```

In the console you can use `window.evaluate` to pass JavaScript in.
To develop on the web assembly side you can run `yarn serve` then go to `http://localhost:8080`.

## Roadmap

See [Milestones](https://github.com/boa-dev/boa/milestones).

## Changelog

See [CHANGELOG.md](./CHANGELOG.md).

## Usage

- Clone this repo.
- Run with `cargo run -- test.js` where `test.js` is an existing JS file.
- If any JS doesn't work then it's a bug. Please raise an issue!

## Profiling

See [Profiling](./docs/profiling.md)

## Command-line Options

```
USAGE:
    boa [OPTIONS] [FILE]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --dump-ast <FORMAT>       Dump the abstract syntax tree (ast) to stdout with the given format [possible values: Debug, Json,
                                  JsonPretty]

ARGS:
    <FILE>...    The JavaScript file(s) to be evaluated
```

## Communication

Feel free to contact us on [Discord](https://discord.gg/tUFFk9Y).

## Example

![Example](docs/img/latestDemo.gif)

## License

This project is licensed under the [Unlicense](./LICENSE-UNLICENSE) or [MIT](./LICENSE-MIT) licenses, at your option.
