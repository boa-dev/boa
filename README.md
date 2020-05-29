# Boa

<p align="center">
    <img
      alt="logo"
      src="./assets/logo.svg"
      width="30%"
    />
</p>

This is an experimental Javascript lexer, parser and compiler written in Rust.
Currently, it has support for some of the language.

[![Build Status](https://travis-ci.com/jasonwilliams/boa.svg?branch=master)](https://travis-ci.com/jasonwilliams/boa)
[![](http://meritbadge.herokuapp.com/boa)](https://crates.io/crates/boa)
[![](https://docs.rs/Boa/badge.svg)](https://docs.rs/Boa/)

## Live Demo (WASM)

<https://boa-dev.github.io/boa/>

You can get more verbose errors when running from the command line.

## Development documentation

You can check the internal development docs at <https://boa-dev.github.io/boa/doc>.

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

## Command-line Options

```
USAGE:
    boa_cli [OPTIONS] [FILE]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --dump-ast <FORMAT>       Dump the ast to stdout with the given format [possible values: Debug, Json,
                                  JsonPretty]
    -t, --dump-tokens <FORMAT>    Dump the token stream to stdout with the given format [possible values: Debug, Json,
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
