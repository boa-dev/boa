# Contributing to Boa

Boa welcomes contribution from everyone. Here are the guidelines if you are
thinking of helping out:

## Contributions

Contributions to Boa or its dependencies should be made in the form of GitHub
pull requests. Each pull request will be reviewed by a core contributor
(someone with permission to land patches) and either landed in the main tree or
given feedback for changes that would be required. All contributions should
follow this format.

Should you wish to work on an issue, please claim it first by commenting on
the GitHub issue that you want to work on it. This is to prevent duplicated
efforts from contributors on the same issue.

Head over to [issues](https://github.com/jasonwilliams/boa/issues) and check for "good first issue" labels to find
good tasks to start with. If you come across words or jargon that do not make
sense, please ask!

If you don't already have Rust installed rustup is the recommended tool to use. It will install Rust and allow you to switch between nightly, stable and beta. You can also install additional components.

`curl https://sh.rustup.rs -sSf | sh`

Then simply clone this project and `cargo build`

### Debugging

You can see the output of tokens by adding a `dbg!(&tokens);` here:
https://github.com/jasonwilliams/boa/blob/master/src/lib/lib.rs#L31

This is useful to know if the tokens are in the right order, or any unexpected tokens are appearing.

The parser's expression tree can be viewed by adding `dbg!(&expr)` here:
https://github.com/jasonwilliams/boa/blob/master/src/lib/lib.rs#L34

To get a full backtrace you will need to set the environment variable `RUST_BACKTRACE=1`


### Web Assembly

If you want to develop on the web assembly side you can run yarn serve then go to http://localhost:8080

### Setup

VScode is commonly used with the [Rust (RLS) plugin](https://github.com/rust-lang/rls-vscode).
