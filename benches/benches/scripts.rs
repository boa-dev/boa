#![allow(unused_crate_dependencies, missing_docs)]
use boa_engine::{
    Context, JsValue, Source, js_string, optimizer::OptimizerOptions, script::Script,
};
use criterion::{Criterion, criterion_group, criterion_main};
use std::path::Path;

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn bench_scripts(c: &mut Criterion) {
    let scripts_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts");

    let scripts: Vec<_> = walkdir::WalkDir::new(&scripts_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "js"))
        .collect();

    for entry in scripts {
        let path = entry.path();
        let code = std::fs::read_to_string(path).unwrap();

        // Create a nice benchmark name from the relative path
        let name = path
            .strip_prefix(&scripts_dir)
            .unwrap()
            .with_extension("")
            .display()
            .to_string();

        let context = &mut Context::default();

        // Disable optimizations
        context.set_optimizer_options(OptimizerOptions::empty());

        // Register runtime for console.log support
        boa_runtime::register(
            boa_runtime::extensions::ConsoleExtension(boa_runtime::NullLogger),
            None,
            context,
        )
        .expect("Runtime registration failed");

        // Parse and compile once, outside the benchmark loop
        let script = Script::parse(Source::from_bytes(&code), None, context).unwrap();
        script.codeblock(context).unwrap();

        // Evaluate once to define the main function
        script.evaluate(context).unwrap();

        // Get the main function
        let function = context
            .global_object()
            .get(js_string!("main"), context)
            .unwrap_or_else(|_| panic!("No main function defined in script: {}", path.display()))
            .as_callable()
            .unwrap_or_else(|| panic!("'main' is not a function in script: {}", path.display()))
            .clone();

        c.bench_function(&format!("{name} (Execution)"), |b| {
            b.iter(|| function.call(&JsValue::undefined(), &[], context));
        });
    }
}

criterion_group!(benches, bench_scripts);
criterion_main!(benches);
