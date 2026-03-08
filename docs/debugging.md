## Debugging

There are multiple ways to debug what Boa is doing. You may also want to
inspect how the engine works internally or simply test some JavaScript code.

One way to do so is to create a file in the root of the repository. For example
test.js. Then execute:

cargo run -- test.js

to run the file with Boa.

You can also execute multiple JavaScript files:

cargo run -- file1.js file2.js

Additionally, Boa can be run interactively by simply calling:

cargo run

This starts the REPL shell where you can execute JavaScript directly.

The following sections describe the debugging tools available in Boa in roughly
the order that the engine processes JavaScript code.

## Tokens and AST nodes

The first step Boa performs is generating tokens from the source code.
These tokens are then parsed into an Abstract Syntax Tree (AST).
Any syntax errors should occur during this stage.

You can use the boa_cli command-line flag --dump-ast to print the AST.

Supported formats:

Debug

Json

JsonPretty

The default format is Debug.

Dumping the AST of a file:

cargo run -- test.js --dump-ast

Using interactive mode (REPL):

cargo run -- --dump-ast

## Bytecode generation and execution

Once the AST has been generated, Boa compiles it into bytecode which is then
executed by the virtual machine (VM).

You can print the bytecode and executed instructions using the --trace flag.

For more detailed information about the VM and trace output see:
vm.md.

## Instruction flowgraph

Boa can generate a visual representation of VM instruction flow.

In the generated graph:

Start (green) represents the start of execution.

End (red) represents the end of execution.

Conditional instructions are diamond-shaped.

"YES" branches are shown in green and "NO" branches in red.

Push/pop environment pairs share colors and are connected by dotted lines.

Use the --flowgraph flag to generate a flowgraph.

Example:

cargo run -- test.js --flowgraph | dot -Tpng > test.png

The output is in Graphviz format by default.

You can also generate Mermaid diagrams:

cargo run -- test.js --flowgraph=mermaid

To change graph direction:

--flowgraph-direction=left-to-right

The default direction is top-to-bottom.

Mermaid diagrams can be rendered directly on GitHub.

## Debugging through the debug object $boa

Some debugging actions are difficult or impossible from standard JavaScript.
For this purpose Boa provides a special debug object called $boa.

This object becomes available when running the CLI with:

cargo run -- --debug-object

It injects the $boa object into the global scope.

The object contains modules such as:

gc

function

object

Example: forcing a garbage collection

$boa.gc.collect()

Tracing a single function:

$boa.function.trace(func, this, ...args)

This allows tracing a specific function without enabling the global --trace
flag which traces the entire program.

Full documentation for $boa can be found here:

boa_object.md

## Compiler panics

If the compiler panics, you can enable a full backtrace by setting the
environment variable:

RUST_BACKTRACE=1

Example:

RUST_BACKTRACE=1 cargo run -- test.js

## Running tests with debug output

When debugging engine behavior it is often useful to run tests while
seeing debug output.

Rust hides println! output in tests by default. To display it, run:

cargo test -- --nocapture

To run a single test with visible output:

cargo test test_name -- --nocapture

This is helpful when investigating specific failing tests.

## Inspecting JsValue internals

When debugging engine behavior it can be useful to inspect JavaScript
values at runtime.

Boa provides helper display functions:

value.display()

To include internal object information such as hidden properties and
prototype details:

value.display().internals(true)

These helpers are frequently used in the REPL and during engine debugging.

## Formatting and linting

Before submitting a pull request or while iterating on debugging changes,
it is recommended to run the project's formatting and linting tools.

Format the code:

cargo fmt

Run the linter:

cargo clippy

These tools help maintain code quality and consistency.

## Debugger VS Code debugger

The easiest way to debug Boa is by using the CodeLLDB extension
for Visual Studio Code and setting breakpoints.

More information is available here:

## LLDB manual debugging

You can also debug manually using rust-lldb:

rust-lldb ./target/debug/boa [arguments]

## Building Boa in debug mode

During development Boa is usually built in debug mode:

cargo build

Debug builds include additional checks and produce more useful stack
traces during debugging.

For performance testing or production builds you can use:

cargo build --release