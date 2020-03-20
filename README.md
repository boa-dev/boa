# Boa

This is an experimental Javascript lexer, parser and compiler written in Rust. Currently, it has support for some of the language.
[![Build Status](https://travis-ci.com/jasonwilliams/boa.svg?branch=master)](https://travis-ci.com/jasonwilliams/boa)
[![](http://meritbadge.herokuapp.com/boa)](https://crates.io/crates/boa)
[![](https://docs.rs/Boa/badge.svg)](https://docs.rs/Boa/)

## Live Demo (WASM)

https://jasonwilliams.github.io/boa/

You can get more verbose errors when running from the command line

## Benchmarks

https://jasonwilliams.github.io/boa/dev/bench/

## Contributing

If you don't already have Rust installed rustup is the recommended tool to use. It will install Rust and allow you to switch between nightly, stable and beta. You can also install additional components.

```
curl https://sh.rustup.rs -sSf | sh
```

Then simply clone this project and `cargo build` inside the directory.

### VSCode

#### Plugins

Either the [Rust (RLS)](https://github.com/rust-lang/rls) or the [Rust Analyzer](https://github.com/rust-analyzer/rust-analyzer) extension is preferred. RLS is easier to set up but most of the development is moving towards Rust Analyzer.
Both of these plugins will help you with your Rust Development

#### Tasks

There are some pre-defined tasks in [tasks.json](.vscode/tasks.json)

- Build - shift+cmd/ctrl+b should build and run cargo. You should be able to make changes and run this task.
- Test - (there is no shortcut, you'll need to make one) - Runs `Cargo Test`.
  I personally set a shortcut of shift+cmd+option+T (or shift+ctrl+alt+T)

If you don't want to install everything on your machine, you can use the Dockerfile.
Start VSCode in container mode (you may need the docker container plugin) and use the Dockerfile.

### Debugging

See [Debugging](./docs/debugging.md)

### Web Assembly

This interpreter can be exposed to javascript!
You can build the example locally with:

```
$ yarn install
$ yarn serve
```

In the console you can use `window.evaluate` to pass JavaScript in
To develop on the web assembly side you can run `yarn serve` then go to `http://localhost:8080`

## Roadmap

See Milestones

## Changelog

see [CHANGELOG](./CHANGELOG.md)

## Usage

- Clone this repo
- Run with `cargo run -- test.js` where `test.js` is an existing JS file
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

Feel free to contact us on Discord https://discord.gg/tUFFk9Y

## Example

![Example](docs/img/latestDemo.gif)

## License

This project is licensed under the Unlicense or MIT licenses, at your option.
