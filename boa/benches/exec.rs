//! Benchmarks of the whole execution engine in Boa.

use boa::{realm::Realm, syntax::Parser, Context};
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[cfg_attr(
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
    global_allocator
)]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn setup_test<'a>(
    name: &'static str,
    js_contents: &'a str,
) -> impl Fn(&'a mut criterion::Criterion) {
    move |c: &'a mut Criterion| {
        let mut group = c.benchmark_group(name);

        // Parse the AST nodes.
        let statement_list = Parser::new(js_contents.as_bytes(), false)
            .parse_all()
            .unwrap();

        group.bench_with_input(BenchmarkId::new(name, 10), &statement_list, move |b, s| {
            b.iter(move || Context::compile(s.to_owned()))
        });

        group.bench_function(&format!("Compile - {}", name), |b| {
            b.iter_batched(
                || {
                    Parser::new(js_contents.as_bytes(), false)
                        .parse_all()
                        .unwrap()
                },
                Context::compile,
                BatchSize::SmallInput,
            );
        });

        group.bench_function(&format!("Run - {}", name), |b| {
            let mut context = Context::new();
            b.iter_batched(
                || Context::compile(statement_list.clone()),
                |input| context.execute(input),
                BatchSize::SmallInput,
            );
        });

        group.finish();
    }
}

fn create_realm(c: &mut Criterion) {
    c.bench_function("Create Realm", move |b| b.iter(Realm::create));
}

// static SYMBOL_CREATION: &str = include_str!("bench_scripts/symbol_creation.js");

// fn symbol_creation<'a>(c: &mut Criterion) -> impl Fn(&'a mut criterion::Criterion) {
//     setup_test("Symbols", SYMBOL_CREATION)
// }

criterion_group!(execution, create_realm);
criterion_main!(execution);
