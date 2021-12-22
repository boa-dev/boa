//! Benchmarks of the whole execution engine in Boa.

use boa::{parse, realm::Realm, Context};
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

macro_rules! exec_bench {
    ($id:literal, $name:ident) => {
        fn $name(c: &mut Criterion) {
            static CODE: &str = include_str!(concat!("bench_scripts/", stringify!($name), ".js"));
            let statement_list = parse(CODE, false).unwrap();
            let code_block = Context::compile(statement_list);
            let mut context = Context::new();
            c.bench_function($id, move |b| {
                b.iter(|| {
                    context.execute(black_box(code_block.clone())).unwrap();
                })
            });
        }
    };
}

exec_bench!("Symbols (Execution)", symbol_creation);
exec_bench!("For loop (Execution)", for_loop);
exec_bench!("Fibonacci (Execution)", fibonacci);
exec_bench!("Object Creation (Execution)", object_creation);
exec_bench!(
    "Static Object Property Access (Execution)",
    object_prop_access_const
);
exec_bench!(
    "Dynamic Object Property Access (Execution)",
    object_prop_access_dyn
);
exec_bench!(
    "RegExp Literal Creation (Execution)",
    regexp_literal_creation
);
exec_bench!("RegExp (Execution)", regexp_creation);
exec_bench!("RegExp Literal (Execution)", regexp_literal);
exec_bench!("RegExp (Execution)", regexp);
exec_bench!("Array access (Execution)", array_access);
exec_bench!("Array creation (Execution)", array_create);
exec_bench!("Array pop (Execution)", array_pop);
exec_bench!("String concatenation (Execution)", string_concat);
exec_bench!("String comparison (Execution)", string_compare);
exec_bench!("String copy (Execution)", string_copy);
exec_bench!("Number Object Access (Execution)", number_object_access);
exec_bench!("Boolean Object Access (Execution)", boolean_object_access);
exec_bench!("String Object Access (Execution)", string_object_access);
exec_bench!("Arithmetic operations (Execution)", arithmetic_operations);
exec_bench!("Clean js (Execution)", clean_js);
exec_bench!("Mini js (Execution)", mini_js);

criterion_group!(
    execution,
    create_realm,
    symbol_creation,
    for_loop,
    fibonacci,
    array_access,
    array_create,
    array_pop,
    object_creation,
    object_prop_access_const,
    object_prop_access_dyn,
    regexp_literal_creation,
    regexp_creation,
    regexp_literal,
    regexp,
    string_concat,
    string_compare,
    string_copy,
    number_object_access,
    boolean_object_access,
    string_object_access,
    arithmetic_operations,
    clean_js,
    mini_js,
);
criterion_main!(execution);
