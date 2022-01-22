//! Benchmarks of the whole execution engine in Boa.

use boa::{realm::Realm, syntax::Parser, Context};
use boa_interner::Interner;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[cfg_attr(
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
    global_allocator
)]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn create_realm(c: &mut Criterion) {
    c.bench_function("Create Realm", move |b| b.iter(Realm::create));
}

macro_rules! full_benchmarks {
    ($({$id:literal, $name:ident}),*) => {
        fn bench_parser(c: &mut Criterion) {
            $(
                {
                    static CODE: &str = include_str!(concat!("bench_scripts/", stringify!($name), ".js"));
                    let mut interner = Interner::new();
                    c.bench_function(concat!($id, " (Parser)"), move |b| {
                        b.iter(|| Parser::new(black_box(CODE.as_bytes()), false).parse_all(&mut interner))
                    });
                }
            )*
        }
        fn bench_compile(c: &mut Criterion) {
            $(
                {
                    static CODE: &str = include_str!(concat!("bench_scripts/", stringify!($name), ".js"));
                    let mut interner = Interner::new();
                    let statement_list = Parser::new(CODE.as_bytes(), false).parse_all( &mut interner).expect("parsing failed");
                    c.bench_function(concat!($id, " (Compiler)"), move |b| {
                        b.iter(|| {
                            Context::compile(black_box(statement_list.clone()));
                        })
                    });
                }
            )*
        }
        fn bench_execution(c: &mut Criterion) {
            $(
                {
                    static CODE: &str = include_str!(concat!("bench_scripts/", stringify!($name), ".js"));
                    let mut interner = Interner::new();
                    let statement_list = Parser::new(CODE.as_bytes(), false).parse_all( &mut interner).expect("parsing failed");
                    let code_block = Context::compile(statement_list);
                    let mut context = Context::default();
                    c.bench_function(concat!($id, " (Execution)"), move |b| {
                        b.iter(|| {
                            context.execute(black_box(code_block.clone())).unwrap();
                        })
                    });
                }
            )*
        }
    };
}

full_benchmarks!(
    {"Symbols", symbol_creation},
    {"For loop", for_loop},
    {"Fibonacci", fibonacci},
    {"Object Creation", object_creation},
    {"Static Object Property Access", object_prop_access_const},
    {"Dynamic Object Property Access", object_prop_access_dyn},
    {"RegExp Literal Creation", regexp_literal_creation},
    {"RegExp Creation", regexp_creation},
    {"RegExp Literal", regexp_literal},
    {"RegExp", regexp},
    {"Array access", array_access},
    {"Array creation", array_create},
    {"Array pop", array_pop},
    {"String concatenation", string_concat},
    {"String comparison", string_compare},
    {"String copy", string_copy},
    {"Number Object Access", number_object_access},
    {"Boolean Object Access", boolean_object_access},
    {"String Object Access", string_object_access},
    {"Arithmetic operations", arithmetic_operations},
    {"Clean js", clean_js},
    {"Mini js", mini_js}
);

criterion_group!(
    benches,
    create_realm,
    bench_parser,
    bench_compile,
    bench_execution,
);
criterion_main!(benches);
