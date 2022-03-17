# Fuzzing

Fuzzing is a process which allows us to identify and test unusual cases that may not be discovered by manual review.
This can be used to identify cases which may cause a crash, incorrectness, or, at worst, a security threat.

## Setup

You'll need to install cargo-fuzz: `cargo install cargo-fuzz`

You may optionally wish to use a corpus; you will need to have generated this corpus from previous executions of the
fuzzer you intend to use, as these fuzzers are limited to Arbitrary-derived data. [There is currently a PR in process to
allow us to convert from JS source back to Arbitrary](https://github.com/rust-fuzz/arbitrary/pull/94).

## Picking a fuzzer

There are currently two fuzzers available: syntax_fuzzer and interp_fuzzer.

### syntax_fuzzer

As the name suggests, this fuzzer is designed to identify issues in the syntax processing phase of Boa. You can run the
fuzzer with the following command from the root of the project:

```shell
cargo fuzz run -s none syntax_fuzzer -- -timeout=5
```

This will execute the fuzzer without sanitisation and a 5-second timeout per test case. In most cases, this will be
sufficient as Rust prevents much of the memory safety violations that would be detected by sanitisation. You can review
options provided to you by cargo-fuzz with `cargo fuzz run --help` and the options provided by libfuzzer with
`cargo fuzz run -s none interp_fuzzer -- -help=1`.

### interp_fuzzer

This fuzzer is designed to identify issues in the VM of Boa, but it will also detect issues in the syntax processing.
If you just want to fuzz Boa whatsoever, you probably want this fuzzer.

You can run the fuzzer with the following command from the root of the project:

```shell
cargo fuzz run -s none interp_fuzzer -- -timeout=5
```

This will execute the fuzzer without sanitisation and a 5-second timeout per test case. In most cases, this will be
sufficient as Rust prevents much of the memory safety violations that would be detected by sanitisation. You can review
options provided to you by cargo-fuzz with `cargo fuzz run --help` and the options provided by libfuzzer with
`cargo fuzz run -s none interp_fuzzer -- -help=1`.

## Parallelisation

You may wish to use multiple cores to fuzz. In that case, you only need to add `--jobs`. As an example, the following
command executes the interpreter fuzzer with 8 threads:

```shell
cargo fuzz run -s none interp_fuzzer --jobs 8
```

You should not execute the fuzzer in excess of the number of CPUs on your system. Make sure to review that your level
of parallelisation is not being restricted by overhead from disk, which may occur with high CPU counts.

## Using an existing corpus

Someone's given you their copy of their corpus. Great! You should put the corpus entries (which will each be named as
the SHA-1 hash of their content) in a folder under `fuzz/corpus/<fuzzer>`, replacing `<fuzzer>` with your intended
fuzz target (either interp_fuzzer or syntax_fuzzer).
