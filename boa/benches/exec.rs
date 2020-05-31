//! Benchmarks of the whole execution engine in Boa.

use boa::{exec::Interpreter, realm::Realm, Executable, Lexer, Parser};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[cfg_attr(
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
    global_allocator
)]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

static SYMBOL_CREATION: &str = r#"
(function () {
    return Symbol();
})();
"#;

fn create_realm(c: &mut Criterion) {
    c.bench_function("Create Realm", move |b| b.iter(Realm::create));
}

fn symbol_creation(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Lex all the tokens.
    let mut lexer = Lexer::new(black_box(SYMBOL_CREATION));
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("Symbols (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static FOR_LOOP: &str = r#"
(function () {
    let b = "hello";
    for (let a = 10; a < 100; a += 5) {
        if (a < 50) {
            b += "world";
        }
    }

    return b;
})();
"#;

fn for_loop_execution(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Lex all the tokens.
    let mut lexer = Lexer::new(black_box(FOR_LOOP));
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("For loop (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static FIBONACCI: &str = r#"
(function () {
    let num = 12;

    function fib(n) {
        if (n <= 1) return 1;
        return fib(n - 1) + fib(n - 2);
    }

    return fib(num);
})();
"#;

fn fibonacci(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Lex all the tokens.
    let mut lexer = Lexer::new(black_box(FIBONACCI));
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("Fibonacci (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

criterion_group!(
    execution,
    create_realm,
    symbol_creation,
    for_loop_execution,
    fibonacci
);
criterion_main!(execution);
