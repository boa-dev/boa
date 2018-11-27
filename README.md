### Boa
This is an experimental Javascript lexer, parser and Just-in-Time compiler written in Rust. Currently, it has support for some of the language.   
[![Build Status](https://travis-ci.com/jasonwilliams/boa.svg?branch=master)](https://travis-ci.com/jasonwilliams/boa)
[![](http://meritbadge.herokuapp.com/boa)](https://crates.io/crates/boa)
[![](https://docs.rs/Boa/badge.svg)](https://docs.rs/Boa/)


This project is an attempted rewrite of Bebbington's js.rs. Most of the Rust code has been rewritten from scratch.

#### Roadmap
* Boxing/Unboxing of primitive types (using "string".length)
* Adding support for constructors
* Better error output
* 

#### Usage
* Checkout this project
* Build `cargo build`
* `cargo run`
* You can make changes to tests/js/test.js and build again
