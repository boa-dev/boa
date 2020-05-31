# Profiling

![Example](img/profiler.png)

It's possible to get a full profile of Boa in action.  
Sometimes this is needed to figure out where it is spending most of it's time.

We use a crate called [measureme](https://github.com/rust-lang/measureme), which helps us keep track of timing functions during runtime.

When the "profiler" flag is enabled, you compile with the profiler and it is called throughout the interpreter.  
when the feature flag is not enabled, you have an empty dummy implementation that is just no ops. rustc should completely optimize that away. So there should be no performance downgrade from these changes

## Prerequesites

- [Crox](https://github.com/rust-lang/measureme/blob/master/crox/Readme.md)

## How To Use

You can run boa using the "profiler" feature flag to enable profiling. Seeing as you'll most likely be using boa_cli you can pass this through, like so:

`cargo run --features Boa/profiler ../tests/js/test.js`

Once finished you should see some trace files left in the directory (boa_cli in this case).  
In the same directory as the `.events, string_data, string_index` files run `crox my_trace` or whatever the name of the files are. This will generate a chrome_profiler.json file, you can load this into Chrome Dev tools.

## More Info

- https://blog.rust-lang.org/inside-rust/2020/02/25/intro-rustc-self-profile.html
- https://github.com/rust-lang/measureme
- https://github.com/rust-lang/measureme/blob/master/crox/Readme.md
