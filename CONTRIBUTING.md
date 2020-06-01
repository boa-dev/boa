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

```
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

## Communication

We have a Discord server, feel free to ask questions here:
https://discord.gg/tUFFk9Y

[issues]: https://github.com/boa-dev/boa/issues
[rustup]: https://rustup.rs/
[rls_vscode]: https://marketplace.visualstudio.com/items?itemName=rust-lang.rust
[rust-analyzer_vscode]: https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer
