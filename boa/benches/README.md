# Boa Benchmarks.

We divide the benchmarks in 3 sections:

- Full engine benchmarks (lexing + parsing + realm creation + execution)
- Execution benchmarks
- Parsing benchmarks (lexing + parse - these are tightly coupled so must be benchmarked together)

The idea is to check the performance of Boa in different scenarios and dividing the Boa execution
process in its different parts.
