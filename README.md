### Boa

This is an experimental Javascript lexer, parser and Just-in-Time compiler written in Rust. Currently, it has support for some of the language.  
[![Build Status](https://travis-ci.com/jasonwilliams/boa.svg?branch=master)](https://travis-ci.com/jasonwilliams/boa)
[![](http://meritbadge.herokuapp.com/boa)](https://crates.io/crates/boa)
[![](https://docs.rs/Boa/badge.svg)](https://docs.rs/Boa/)

This project is an attempted rewrite of Bebbington's js.rs. Most of the Rust code has been rewritten from scratch.

#### Live Demo

https://jasonwilliams.github.io/boa/

You can get more verbose errors when running from the commnand line

### Contributing

If you don't already have Rust installed rustup is the recommended tool to use. It will install Rust and allow you to switch between nightly, stable and beta. You can also install additional components.

```
curl https://sh.rustup.rs -sSf | sh
```

Then simply clone this project and `cargo build`
To develop on the web assembly side you can run `yarn serve` then go to `http://localhost:8080`

#### Web Assembly

This interpreter can be exposed to javascript!
You can build the example locally with:

```
$ yarn install
$ yarn serve
```

In the console you can use `window.evaluate` to pass JavaScript in

#### Roadmap

See Project view

#### Usage

- Checkout this project
- Build `cargo build`
- `cargo run`
- You can make changes to tests/js/test.js and build again
- If any JS doesn't work its a bug! Please raise an issue

#### Example

![Example](docs/img/latestDemo.gif)
