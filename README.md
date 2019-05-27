### Boa

This is an experimental Javascript lexer, parser and Just-in-Time compiler written in Rust. Currently, it has support for some of the language.  
[![Build Status](https://travis-ci.com/jasonwilliams/boa.svg?branch=master)](https://travis-ci.com/jasonwilliams/boa)
[![](http://meritbadge.herokuapp.com/boa)](https://crates.io/crates/boa)
[![](https://docs.rs/Boa/badge.svg)](https://docs.rs/Boa/)

This project is an attempted rewrite of Bebbington's js.rs. Most of the Rust code has been rewritten from scratch.

#### WASM

This interpreter can be exposed to javascript!
You can build the example locally with:

```
$ yarn install
$ yarn serve
```

In the console you can use `window.evaluate` to pass JavaScript in

#### Roadmap

- ~string.length~ - works in 0.1.5
- Adding support for constructors - half working, in progress
- ~better environment and scope support~ - Finished (05/05/2019)
- ~`let` and `const` support~
- Better error output
- Passing [test262](https://github.com/tc39/test262)

#### Usage

- Checkout this project
- Build `cargo build`
- `cargo run`
- You can make changes to tests/js/test.js and build again
- If any JS doesn't work its a bug! Please raise an issue

#### Example

![Example](docs/img/latestDemo.gif)
