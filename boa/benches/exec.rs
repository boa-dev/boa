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

    // Parse the AST nodes.
    let nodes = Parser::new(black_box(SYMBOL_CREATION.as_bytes()))
        .parse_all()
        .unwrap();

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

    // Parse the AST nodes.
    let nodes = Parser::new(black_box(FOR_LOOP.as_bytes()))
        .parse_all()
        .unwrap();

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

    // Parse the AST nodes.
    let nodes = Parser::new(black_box(FIBONACCI.as_bytes()))
        .parse_all()
        .unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("Fibonacci (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static OBJECT_CREATION: &str = r#"
(function () {
    let test = {
        my_prop: "hello",
        another: 65,
    };

    return test;
})();
"#;

fn object_creation(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Parse the AST nodes.
    let nodes = Parser::new(black_box(OBJECT_CREATION.as_bytes()))
        .parse_all()
        .unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("Object Creation (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static OBJECT_PROP_ACCESS_CONST: &str = r#"
(function () {
    let test = {
        my_prop: "hello",
        another: 65,
    };

    return test.my_prop;
})();
"#;

fn object_prop_access_const(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Parse the AST nodes.
    let nodes = Parser::new(black_box(OBJECT_PROP_ACCESS_CONST.as_bytes()))
        .parse_all()
        .unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("Static Object Property Access (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static OBJECT_PROP_ACCESS_DYN: &str = r#"
(function () {
    let test = {
        my_prop: "hello",
        another: 65,
    };

    return test["my" + "_prop"];
})();
"#;

fn object_prop_access_dyn(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Parse the AST nodes.
    let nodes = Parser::new(black_box(OBJECT_PROP_ACCESS_DYN.as_bytes()))
        .parse_all()
        .unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("Dynamic Object Property Access (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static REGEXP_LITERAL_CREATION: &str = r#"
(function () {
    let regExp = /hello/i;

    return regExp;
})();
"#;

fn regexp_literal_creation(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Parse the AST nodes.
    let nodes = Parser::new(black_box(REGEXP_LITERAL_CREATION.as_bytes()))
        .parse_all()
        .unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("RegExp Literal Creation (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static REGEXP_CREATION: &str = r#"
(function () {
    let regExp = new RegExp('hello', 'i');

    return regExp;
})();
"#;

fn regexp_creation(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Parse the AST nodes.
    let nodes = Parser::new(black_box(REGEXP_CREATION.as_bytes()))
        .parse_all()
        .unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("RegExp (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static REGEXP_LITERAL: &str = r#"
(function () {
    let regExp = /hello/i;

    return regExp.test("Hello World");
})();
"#;

fn regexp_literal(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Parse the AST nodes.
    let nodes = Parser::new(black_box(REGEXP_LITERAL.as_bytes()))
        .parse_all()
        .unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("RegExp Literal (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static REGEXP: &str = r#"
(function () {
    let regExp = new RegExp('hello', 'i');

    return regExp.test("Hello World");
})();
"#;

fn regexp(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Parse the AST nodes.
    let nodes = Parser::new(black_box(REGEXP.as_bytes()))
        .parse_all()
        .unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("RegExp (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

criterion_group!(
    execution,
    create_realm,
    symbol_creation,
    for_loop_execution,
    fibonacci,
    object_creation,
    object_prop_access_const,
    object_prop_access_dyn,
    regexp_literal_creation,
    regexp_creation,
    regexp_literal,
    regexp,
);
criterion_main!(execution);
