#![allow(unused_crate_dependencies, missing_docs)]

use boa_engine::{
    Context, JsValue, Source, js_string, optimizer::OptimizerOptions, script::Script,
};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn bench_string_slice(c: &mut Criterion) {
    static CODE: &str = include_str!("../scripts/strings/slice.js");
    let context = &mut Context::default();

    // Disable optimizations
    context.set_optimizer_options(OptimizerOptions::empty());

    // Register runtime.
    boa_runtime::register(
        boa_runtime::extensions::ConsoleExtension::default(),
        None,
        context,
    )
    .expect("Runtime registration failed");

    // Parse and compile once, outside the benchmark loop
    let script = Script::parse(black_box(Source::from_bytes(CODE)), None, context).unwrap();
    script.codeblock(context).unwrap();

    script.evaluate(context).unwrap();

    // Get the benched function in.
    let function = context
        .global_object()
        .get(js_string!("main"), context)
        .expect("No main function defined in script")
        .as_function()
        .unwrap();

    c.bench_function("String slice (Execution)", move |b| {
        b.iter(|| function.call(&JsValue::undefined(), &[], context));
    });
}

criterion_group!(benches, bench_string_slice);
criterion_main!(benches);
