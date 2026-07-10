# Bytecode Snapshot tests

This crate includes snapshot tests for Boa's bytecode. This gives us a workable diff
of the bytecode output.

Required dependency: `cargo-insta`

## Reviewing snapshots

Snapshots can be reviewed with the below command.

```bash
cargo insta test --review
```

OR

```bash
cargo insta review
```
