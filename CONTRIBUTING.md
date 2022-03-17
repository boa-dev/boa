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

Head over to [issues][issues] and check for "good first issue" labels to find
good tasks to start with. If you come across words or jargon that do not make
sense, please ask!

If you don't already have Rust installed [_rustup_][rustup] is the recommended
tool to use. It will install Rust and allow you to switch between _nightly_,
_stable_ and _beta_. You can also install additional components. In Linux, you
can run:

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then simply clone this project and `cargo build`.

### Running the compiler

You can execute a Boa console by running `cargo run`, and you can compile a list
of JavaScript files by running `cargo run -- file1.js file2.js` and so on.

### Debugging

Knowing how to debug the interpreter should help you resolve problems quite quickly.
See [Debugging](./docs/debugging.md).

### Web Assembly

If you want to develop on the web assembly side you can run `yarn serve` and then go
to <http://localhost:8080>.

### boa-unicode

Boa uses the library `boa-unicode` to query Unicode character properties and classes in lexer and parser. See [boa_unicode/README.md](./boa_unicode/README.md) for development and more information.

### Setup

#### VSCode Plugins

Either the [Rust (RLS)][rls_vscode] or the [Rust Analyzer][rust-analyzer_vscode]
extensions are preferred. RLS is easier to set up but some of the development is
moving towards Rust Analyzer. Both of these plugins will help you with your Rust
Development

#### Tasks

There are some pre-defined tasks in [tasks.json](.vscode/tasks.json)

- Build - shift+cmd/ctrl+b should build and run cargo. You should be able to make changes and run this task.
- Test - (there is no shortcut, you'll need to make one) - Runs `Cargo Test`.
  I personally set a shortcut of shift+cmd+option+T (or shift+ctrl+alt+T)

If you don't want to install everything on your machine, you can use the Dockerfile.
Start VSCode in container mode (you may need the docker container plugin) and use the Dockerfile.

## Testing

Boa provides its own test suite, and can also run the official ECMAScript test suite. To run the Boa test
suite, you can just run the normal `cargo test`, and to run the full ECMAScript test suite, you can run it
with this command:

```shell
cargo run --release --bin boa_tester -- run -v 2> error.log
```

Note that this requires the `test262` submodule to be checked out, so you will need to run the following first:

```shell
git submodule init && git submodule update
```

This will run the test suite in verbose mode (you can remove the `-v` part to run it in non-verbose mode),
and output nice colorings in the terminal. It will also output any panic information into the `error.log` file.

You can get some more verbose information that tells you the exact name of each test that is being run, useful
for debugging purposes by setting up the verbose flag twice, for example `-vv`. If you want to know the output of
each test that is executed, you can use the triple verbose (`-vvv`) flag.

If you want to only run one sub-suite or even one test (to just check if you fixed/broke something specific),
you can do it with the `-s` parameter, and then passing the path to the sub-suite or test that you want to run. Note
that the `-s` parameter value should be a path relative to the `test262` directory. For example, to run the number
type tests, use `-s test/language/types/number`.

Finally, if you're using the verbose flag and running a sub suite with a small number of tests, then the output will
be more readable if you disable parallelism with the `-d` flag. All together it might look something like:

```shell
cargo run --release --bin boa_tester -- run -vv -d -s test/language/types/number 2> error.log
```

## Communication

We have a Discord server, feel free to ask questions here:
<https://discord.gg/tUFFk9Y>

[issues]: https://github.com/boa-dev/boa/issues
[rustup]: https://rustup.rs/
[rls_vscode]: https://marketplace.visualstudio.com/items?itemName=rust-lang.rust
[rust-analyzer_vscode]: https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer
