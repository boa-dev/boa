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

## Instruction flowgraph

We can to get the vm instructions flowgraph, which is a visual representation of the instruction flow.

The `Start` (in green) and `End` (in red) node in the graph represents the start and end point of execution.
They are not instructions, just markers.

The conditional instructions are diamond shaped, with the `"YES"` branch in green and the `"NO"` branch in red.
The push and pop evironment pairs match colors and are connected by a dotted line.

You can use the `--flowgraph` (or `--flowgraph=mermaid` for [mermaid][mermaid] format) flag which outputs
[graphviz][graphviz] format by default, and pipe it to `dot` (from the `graphviz` package which is installed
on most linux distros by default) or use an online editor like: <https://dreampuf.github.io/GraphvizOnline> to
view the graph.

```bash
cargo run -- test.js --flowgraph | dot -Tpng > test.png
```

You can specify the `-Tsvg` to generate a `svg` instead of a `png` file.

![Graphviz flowgraph](./img/graphviz_flowgraph.svg)

Mermaid graphs can be displayed on github [natively without third-party programs][gihub-mermaid].
By using a `mermaid` block as seen below.

````
```mermaid
// graph contents here...
```
````

Additionaly you can specify the direction of "flow" by using the `--flowgraph-direction` cli option,
for example `--flowgraph-direction=left-to-right`, the default is `top-to-bottom`.

[mermaid]: https://mermaid-js.github.io/
[gihub-mermaid]: https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/creating-diagrams
[graphviz]: https://graphviz.org/

## Debugging through the debug object $boa

Certain debugging actions in JavaScript land are difficult to impossible, like triggering a GC collect.

For such puroposes we have the `$boa` object that contains useful utilities that can be used to debug JavaScript in JavaScript.
The debug object becomes available with the `--debug-object` cli flag, It injects the `$boa` debug object in the context as global variable,
the object is separated into modules `gc`, `function`, `object`, etc.

We can now do `$boa.gc.collect()`, which force triggers a GC collect.

If you want to trace only a particular function (without being flodded by the `--trace` flag, that traces everything),
for that we have the `$boa.function.trace(func, this, ...args)`.

The full documentation of the `$boa` object's modules and functionalities can be found [`here`](./boa_object.md).

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
