#![allow(unused_crate_dependencies, missing_docs)]
use boa_engine::{
    Context, JsValue, Source, js_string, object::JsObject, optimizer::OptimizerOptions,
    script::Script,
};
use criterion::{Criterion, criterion_group, criterion_main};
use std::{path::Path, time::Duration};

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

struct PreparedScriptBench {
    context: Context,
    function: JsObject,
}

fn prepare_script_bench(path: &Path) -> PreparedScriptBench {
    let code = std::fs::read_to_string(path).unwrap();

    let mut context = Context::default();
    context.set_optimizer_options(OptimizerOptions::empty());

    boa_runtime::register(
        boa_runtime::extensions::ConsoleExtension(boa_runtime::NullLogger),
        None,
        &mut context,
    )
    .expect("Runtime registration failed");

    let script = Script::parse(Source::from_bytes(&code), None, &mut context).unwrap();
    script.codeblock(&mut context).unwrap();
    script.evaluate(&mut context).unwrap();

    let function = context
        .global_object()
        .get(js_string!("main"), &mut context)
        .unwrap_or_else(|_| panic!("No main function defined in script: {}", path.display()))
        .as_callable()
        .unwrap_or_else(|| panic!("'main' is not a function in script: {}", path.display()))
        .clone();

    PreparedScriptBench { context, function }
}

fn bench_scripts(c: &mut Criterion) {
    let args: Vec<String> = std::env::args().collect();
    let is_list = args.iter().any(|arg| arg == "--list");
    let filter = args.iter().skip(1).find(|arg| !arg.starts_with('-'));
    let filter_re = filter.and_then(|f| regex::Regex::new(f).ok());

    let scripts_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts");

    let scripts: Vec<_> = walkdir::WalkDir::new(&scripts_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.extension().is_some_and(|ext| ext == "js")
                && path
                    .file_name()
                    .is_some_and(|base| !base.display().to_string().starts_with("_"))
        })
        .collect();

    for entry in scripts {
        let path = entry.path();

        let rel_path = path.strip_prefix(&scripts_dir).unwrap().with_extension("");
        let name = rel_path.display().to_string();

        let mut group = c.benchmark_group(&name);
        if rel_path.starts_with("v8-benches") {
            group.sample_size(10);
            group.measurement_time(Duration::from_secs(5));
        }

        let should_run = !is_list
            && filter.is_none_or(|f| {
                let full_name = format!("{name}/Execution");
                if let Some(re) = &filter_re {
                    re.is_match(&full_name)
                } else {
                    full_name.contains(f)
                }
            });

        let mut prepared = should_run.then(|| prepare_script_bench(path));

        group.bench_function("Execution", move |b| {
            if let Some(prepared) = prepared.as_mut() {
                let function = prepared.function.clone();
                let context = &mut prepared.context;

                b.iter(|| function.call(&JsValue::undefined(), &[], context));
            }
        });
        group.finish();
    }
}

criterion_group!(benches, bench_scripts);
criterion_main!(benches);
