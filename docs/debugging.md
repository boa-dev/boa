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

## Tokens and AST nodes

The first thing boa will do is to generate tokens from the source code.
These tokens are then parsed into an abstract syntax tree (AST).
Any syntax errors should be thrown while the AST is generated.

You can use the `boa_cli` command-line flag `--dump-ast` to print the AST.
The flag supports these formats: `Debug`, `Json`, `JsonPretty`. By default
it is the `Debug` format.

Dumping the AST of a file:

```bash
cargo run -- test.js --dump-ast # AST dump format is Debug by default.
```

or with interactive mode (REPL):

```bash
cargo run -- --dump-ast # AST dump format is Debug by default.
```

## Bytecode generation and Execution

Once the AST has been generated boa will compile it into bytecode.
The bytecode is then executed by the vm.
You can print the bytecode and the executed instructions with the command-line flag `--trace`.

For more detailed information about the vm and the trace output look [here](./vm.md).

## Compiler panics

In the case of a compiler panic, to get a full backtrace you will need to set
the environment variable `RUST_BACKTRACE=1`.

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
