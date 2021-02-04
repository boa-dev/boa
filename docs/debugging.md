# Debugging

There are multiple ways to debug what Boa is doing. Or maybe you just want to
know how it works under the hood. Or even test some JavaScript.

One way to do so is to create a file in the root of the repository. For example
`test.js`. Then execute `cargo run -- test.js` to run the file with boa. You can
compile a list of JavaScript files by running `cargo run -- file1.js file2.js`
and so on.

You can also run boa interactively by simply calling `cargo run` without any
arguments to start a shell to execute JS.

These are added in order of how the code is read:

## Tokens

The first thing boa will do is generate tokens from source code. If the token
generation is wrong the rest of the operation will be wrong, this is usually
a good starting place.

To print the tokens to stdout, you can use the `boa_cli` command-line flag
`--dump-tokens` or `-t`, which can optionally take a format type. Supports
these formats: `Debug`, `Json`, `JsonPretty`. By default it is the `Debug`
format.

```bash
cargo run -- test.js --dump-tokens # token dump format is Debug by default.
```

or with interactive mode (REPL):

```bash
cargo run -- --dump-tokens # token dump format is Debug by default.
```

Seeing the order of tokens can be a big help to understanding what the parser
is working with.

**Note:** flags `--dump-tokens` and `--dump-ast` are mutually exclusive. When
using the flag `--dump-tokens`, the code will not be executed.

## AST nodes

Assuming the tokens looks fine, the next step is to see the AST. You can use
the `boa_cli` command-line flag `--dump-ast`, which can optionally take a
format type. Supports these formats: `Debug`, `Json`, `JsonPretty`. By default
it is the `Debug` format.

Dumping the AST of a file:

```bash
cargo run -- test.js --dump-ast # AST dump format is Debug by default.
```

or with interactive mode (REPL):

```bash
cargo run -- --dump-ast # AST dump format is Debug by default.
```

These methods will print out the entire parse tree.

**Note:** flags `--dump-tokens` and `--dump-ast` are mutually exclusive. When
using the flag `--dump-ast`, the code will not be executed.

## Compiler panics

In the case of a compiler panic, to get a full backtrace you will need to set
the environment variable `RUST_BACKTRACE=1`.

## Execution

Once the tree has been generated [exec](../boa/src/lib.rs#L92) will begin to
run through each node. If the tokens and tree looks fine, you can start looking
here. We usually just add `dbg!()` in the relevent places to see what the
output is at the time.

## Debugger

### VS Code Debugger

The quickest way to get debugging is to use the CodeLLDB plugin and add breakpoints. You can get
more information [here][blog_debugging].

### LLDB Manual debugging

You can also use rust-lldb. The `Dockerfile` already has this enabled, you
should be able to use that environment to run your code.

```
rust-lldb ./target/debug/boa [arguments]
```

[remote_containers]: https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers
[blog_debugging]: https://jason-williams.co.uk/debugging-rust-in-vscode

## VM

For debugging the new VM see [here](./vm.md)
