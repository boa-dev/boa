# Boa CLI

Boa CLI is `Boa`'s REPL implementation to execute `JavaScript` directly from
your CLI.

## Installation

`boa_cli` can be installed directly via `Cargo`.

```shell
    cargo install boa_cli
```

<!-- TODO (nekevss): Add a non cargo-based installation options / build out further -->

## Usage

<!-- TODO (nekevss): Potentially add CI driven gifs with https://github.com/charmbracelet/vhs -->
<!-- NOTE (nekevss): VHS is currently bugged and non-functional on Windows. -->

Once installed, your good to go!

To execute some JavaScript source code, navigate to the directory of your choice and type:

```shell
    boa test.js
```

Or if you'd like to use Boa's REPL, simply type:

```shell
    boa
```

You can also pipe JavaScript into Boa:

```shell
    echo 'console.log(1 + 2)' | boa
    cat script.js | boa
    boa < script.js
```

## CLI Options

```txt
Usage: boa [OPTIONS] [FILE]...

Arguments:
  [FILE]...  The JavaScript file(s) to be evaluated

Options:
      --strict                        Run in strict mode
  -a, --dump-ast [<FORMAT>]           Dump the AST to stdout with the given format [possible values: debug, json, json-pretty]
  -t, --trace                         Dump the AST to stdout with the given format
      --vi                            Use vi mode in the REPL
  -O, --optimize
      --optimizer-statistics
      --flowgraph [<FORMAT>]          Generate instruction flowgraph. Default is Graphviz [possible values: graphviz, mermaid]
      --flowgraph-direction <FORMAT>  Specifies the direction of the flowgraph. Default is top-top-bottom [possible values: top-to-bottom, bottom-to-top, left-to-right, right-to-left]
      --debug-object                  Inject debugging object `$boa`
  -m, --module                        Treats the input files as modules
  -r, --root <ROOT>                   Root path from where the module resolver will try to load the modules [default: .]
  -h, --help                          Print help (see more with '--help')
  -V, --version                       Print version
```

## Features

Boa's CLI currently has a variety of features (as listed in `Options`).

Features include:

- Implemented runtime features (please note that only `Console` is currently implemented)
- AST Visibility: View the compiled Boa AST (--dump-ast)
- Tracing: Enabling a vm tracing when executing any JavaScript
- Flowgraphs: View a generated (with various provided options)
- Debugging: Boa's CLI comes with an implemented `$boa` debug object with various functionality (see documentation).

Have an idea for a feature? Feel free to submit an issue and/or contribute!
