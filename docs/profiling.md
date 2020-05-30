# Profiling

![Example](img/profiler.png)

It's possible to get a full profile of Boa in action.  
Sometimes this is needed to figure out where it is spending most of it's time.

We use a crate called [measureme](https://github.com/rust-lang/measureme), which helps us keep track of timing functions during runtime.

## How To Use

You can run boa using the "profiler" feature flag to enable profiling. Seeing as you'll most likely be using boa_cli you can pass this through, like so:

`cargo run --features Boa/profiler ../tests/js/test.js`

## More Info

https://blog.rust-lang.org/inside-rust/2020/02/25/intro-rustc-self-profile.html
