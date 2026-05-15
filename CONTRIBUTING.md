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

## Testing

Boa provides its own test suite, and can also run the official ECMAScript test suite. To run the Boa test
suite, you can just run the normal `cargo test`, and to run the full ECMAScript test suite, you can run it
with this command:

```shell
cargo run --release --bin boa_tester -- run -v 2> error.log
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

To save test results for later comparison, use the `-o` flag to specify an output directory:

```shell
cargo run --release --bin boa_tester -- run -o ./test-results
```

### Comparing Test Results

You can compare two test suite runs to see what changed:

```shell
cargo run --release --bin boa_tester -- compare <base-results> <new-results>
```

Both arguments can be either result files (e.g., `latest.json`) or directories containing test results.
When directories are provided, the tester automatically uses the `latest.json` file from each directory.

For example:

```shell
# Compare using directories
cargo run --release --bin boa_tester -- compare ./test-results-main ./test-results-feature

# Compare using explicit files
cargo run --release --bin boa_tester -- compare ./test-results-main/latest.json ./test-results-feature/latest.json
```

## Documentation

To build the development documentation, run:

```shell
cargo doc --all-features --document-private-items --workspace
```

This will also document all the dependencies on the workspace, which could be heavier in size.
To only generate documentation for the workspace members, just add the `--no-deps` flag:

```shell
cargo doc --all-features --document-private-items --workspace --no-deps
```

## Reading and Understanding the ECMAScript Specification

Many contributions to Boa involve implementing parts of the [ECMAScript
Language Specification](https://tc39.es/ecma262/), which defines how JavaScript
behaves. At first, the spec can seem intimidating, but it quickly becomes
easier to follow once you get familiar with its structure and notation.

The specification is written in a pseudo-language designed to describe behavior
without being tied to any particular programming language. It introduces some
important concepts:

- **Abstract operations** – general algorithms (i.e. [`IsCallable`](https://tc39.es/ecma262/#sec-iscallable)),
  which usually map to Rust functions or methods.
- **Internal slots** – hidden object fields like
  [`[[Prototype]]`](https://tc39.es/ecma262/#sec-ordinary-object-internal-methods-and-internal-slots)
  that correspond to private struct or enum fields in Rust, not accessible to
  JavaScript.
- **Completion records** – describe how values or exceptions are returned
  ([link](https://tc39.es/ecma262/#sec-completion-record-specification-type)),
  and typically map to `JsResult` types in Rust.
- **Symbols `?` and `!`** – `? Foo(...)` propagates exceptions mapped to
  propagate `?` operator in rust, while `! Foo(...)` are infallible operations
  and are usually mapped to
  [`Result::expect()`](https://doc.rust-lang.org/std/result/enum.Result.html#method.expect)
  call.

For an in-depth introduction to these concepts and more, check out [V8’s “Understanding
the ECMAScript spec”
series](https://v8.dev/blog/tags/understanding-ecmascript), starting with [Part
1](https://v8.dev/blog/understanding-ecmascript-part-1).

When implementing the spec in Boa, try to map your code to the corresponding
spec steps whenever possible, and indicate in comments which steps are implemented.
This makes the code easier to understand, ensures it aligns with the
specification, and helps reviewers and future contributors follow the logic.

If a spec step does not map directly because of Rust limitations or performance
reasons, just add a note in the code explaining the difference. Being clear
about these cases helps others understand your implementation while still
following the spec as closely as possible.

For examples of how to implement the specification, check out the built-in
implementations in Boa
[here](https://github.com/boa-dev/boa/tree/main/core/engine/src/builtins).

If anything in the specification is confusing, don’t hesitate to ask in the
[Boa Matrix](https://matrix.to/#/#boa:matrix.org) channel.

## Unsafe Code Guidelines

Boa's core crates use `unsafe` Rust in carefully scoped locations where performance
requirements justify it — primarily in the garbage collector (`boa_gc`), string
representation (`boa_string`), and the NaN-boxed value type. All unsafe code must
follow these guidelines.

### General Principles

1. **Unsafe is a last resort.** Exhaust safe alternatives first. Only use `unsafe`
   when a safe equivalent is measurably slower or impossible in current Rust.
2. **Every `unsafe` block must be justified.** If you cannot explain *why* it's
   safe in a sentence, redesign to avoid the unsafe.
3. **Unsafe code must be tested more thoroughly.** Plan for additional unit tests
   that exercise the unsafe invariants. All unsafe code should be tested with
   [MIRI](https://github.com/rust-lang/miri).

### Required Documentation

#### `unsafe fn` declarations

Every `unsafe fn` must have a `# Safety` section in its doc comment describing the
invariants callers must uphold:

```rust
/// [... description ...]
///
/// # Safety
///
/// - The caller must ensure that `ptr` is non-null, aligned, and points to a
///   valid, initialized `JsBigInt`.
/// - The caller must ensure that no other mutable reference to the same data
///   exists for the lifetime `'a`.
unsafe fn as_bigint_unchecked(&self) -> ManuallyDrop<JsBigInt> { ... }
```

See `core/engine/src/value/inner/nan_boxed.rs` for consistent examples of this pattern.

#### `unsafe { }` blocks

Every `unsafe { }` block must have a preceding `// SAFETY:` comment explaining
**why** the operation is safe at this call site — not just restating what the
unsafe operation does:

```rust
// SAFETY: We verified the tag is `MASK_STRING` via `self.is_string()`,
// so `as_string_unchecked` will return a valid `JsString`.
unsafe { Some((*self.as_string_unchecked()).clone()) }
```

Bad — does not explain why:
```rust
// SAFETY: unsafe but ok.
unsafe { ptr::copy_nonoverlapping(src, dest, count) }
```

Good — explains the invariant:
```rust
// SAFETY: src has at least `count` bytes (allocated at line 120), dest
// was freshly allocated with `count` bytes, and the regions don't overlap
// because `dest` was allocated from a separate arena. Verified by the caller
// through `check_bounds(dest, src, count)` above.
unsafe { ptr::copy_nonoverlapping(src, dest, count) }
```

#### Unsafe match arms

When matching on a discriminant that guarantees a variant's validity, each `unsafe`
arm should document why the discriminant check ensures safety:

```rust
match self.value() & bits::MASK_KIND {
    bits::MASK_STRING => {
        // SAFETY: tag confirmed this is a String.
        unsafe { !self.as_string_unchecked().is_empty() }
    }
    bits::MASK_BIGINT => {
        // SAFETY: tag confirmed this is a BigInt.
        unsafe { ... }
    }
    // ...
}
```

#### `unsafe impl Trait`

Document what contract the implementor must uphold. For the GC's `Trace` trait
(used extensively), explain why the type doesn't contain `Gc` pointers, or how
it correctly traces them:

```rust
/// SAFETY: `JsString` contains no `Gc`-allocated data.
/// All reachable memory is owned by the string's inline buffer or heap allocation,
/// neither of which need GC tracing.
unsafe impl Trace for JsString { ... }
```

### Enforcing with Lints

Enable these lints in any crate that contains unsafe code:

```rust
#![deny(
    unsafe_op_in_unsafe_fn,
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc
)]
```

- `unsafe_op_in_unsafe_fn` — prevents implicit unsafe operations inside `unsafe fn`.
  Every unsafe operation must be wrapped in its own `unsafe { }` block with a
  `// SAFETY:` comment.
- `clippy::undocumented_unsafe_blocks` — ensures no `unsafe { }` block lacks a
  `// SAFETY:` comment.
- `clippy::missing_safety_doc` — ensures every `unsafe fn` has a `# Safety` section.

Currently, `core/string` enforces all three. Other crates with unsafe code should
adopt these lints incrementally.

### Patterns Specific to Boa

#### `#[repr(C)]` for type-punning

Types that participate in pointer erasure (e.g., `GcBox<T>`, `Object<T>`, vtable
types) must be `#[repr(C)]` to guarantee field layout. The corresponding pointer
casts must be documented with the layout invariant:

```rust
// SAFETY: `GcBox<T>` is `#[repr(C)]` with `header` as the first field,
// so casting `*const GcBox<T>` to `*const GcHeader` is valid.
let header = unsafe { &*(this as *const GcBox<T> as *const GcHeader) };
```

#### `unsafe fn` convention: `_unchecked` suffix

Public unsafe accessors that skip runtime validation should use the `_unchecked`
suffix to signal to callers that bounds checking or tag checking is bypassed:

```rust
pub unsafe fn downcast_unchecked<T: Trace + 'static>(self) -> Gc<T>;
pub unsafe fn slice_unchecked(data: &JsString, start: usize, end: usize) -> Self;
```

Safe wrappers that call `_unchecked` after validation must still have a `// SAFETY:`
comment:

```rust
pub fn as_bigint(&self) -> Option<JsBigInt> {
    if self.is_bigint() {
        // SAFETY: `is_bigint()` returned true, so the inner tag is BigInt.
        unsafe { Some((*self.as_bigint_unchecked()).clone()) }
    } else {
        None
    }
}
```

### Testing Unsafe Code

1. **Unit tests must cover error paths.** Test the invariants explicitly — pass
   invalid indices, dangling pointers (in controlled test harnesses), and edge
   cases around the documented safety preconditions.
2. **Run MIRI on all unsafe code.** Boa's CI includes MIRI jobs. Before submitting
   a PR that touches `unsafe`:
   ```shell
   cargo +nightly miri test -p boa_gc
   cargo +nightly miri test -p boa_string
   ```
3. **Document test coverage for each unsafe block.** If a block relies on invariant
   `X`, there should be a test that would fail if `X` is violated.

## Learning Resources

For contributors looking to learn JavaScript and how it works, check out the [Mozilla Developer Guided Tours](https://www.youtube.com/playlist?list=PLo3w8EB99pqJVPhmYbYdInBvAGarDavh-).

## Communication

We have a Matrix space, feel free to ask questions here:
<https://matrix.to/#/#boa:matrix.org>

[issues]: https://github.com/boa-dev/boa/issues
[rustup]: https://rustup.rs/
[rls_vscode]: https://marketplace.visualstudio.com/items?itemName=rust-lang.rust
[rust-analyzer_vscode]: https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer
