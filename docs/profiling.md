# Profiling

![Example](img/profiler.png)

Boa can be profiled to understand where time is being spent during execution. This is useful for optimizing performance or diagnosing bottlenecks in script execution.

We recommend using one of the following tools:

- Flamegraph – generates a visual representation of the call stack, showing which functions consume the most CPU time.

- Valgrind (Callgrind) – provides detailed execution profiles, which can be explored with tools like KCachegrind or QCachegrind.

These tools do not require any special feature flags or instrumentation in the codebase. They work by sampling or tracing the actual execution and provide a consistent view of performance.

## More Info

- https://blog.rust-lang.org/inside-rust/2020/02/25/intro-rustc-self-profile.html
- https://github.com/brendangregg/Flamegraph
- https://valgrind.org/docs/manual/index.html
