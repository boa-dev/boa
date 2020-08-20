//! Benchmarks of the whole execution engine in Boa.

use boa::{exec::Interpreter, realm::Realm, Executable, Lexer, Parser};
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

static SYMBOL_CREATION: &str = include_str!("bench_scripts/symbol_creation.js");

fn symbol_creation(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Lex all the tokens.
    let mut lexer = Lexer::new(SYMBOL_CREATION);
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("Symbols (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static FOR_LOOP: &str = include_str!("bench_scripts/for_loop.js");

fn for_loop_execution(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Lex all the tokens.
    let mut lexer = Lexer::new(FOR_LOOP);
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("For loop (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static FIBONACCI: &str = include_str!("bench_scripts/fibonacci.js");

fn fibonacci(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Lex all the tokens.
    let mut lexer = Lexer::new(FIBONACCI);
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("Fibonacci (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static OBJECT_CREATION: &str = include_str!("bench_scripts/object_creation.js");

fn object_creation(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Lex all the tokens.
    let mut lexer = Lexer::new(OBJECT_CREATION);
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("Object Creation (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static OBJECT_PROP_ACCESS_CONST: &str = include_str!("bench_scripts/object_prop_access_const.js");

fn object_prop_access_const(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Lex all the tokens.
    let mut lexer = Lexer::new(OBJECT_PROP_ACCESS_CONST);
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("Static Object Property Access (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static OBJECT_PROP_ACCESS_DYN: &str = include_str!("bench_scripts/object_prop_access_dyn.js");

fn object_prop_access_dyn(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Lex all the tokens.
    let mut lexer = Lexer::new(OBJECT_PROP_ACCESS_DYN);
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("Dynamic Object Property Access (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static REGEXP_LITERAL_CREATION: &str = include_str!("bench_scripts/regexp_literal_creation.js");

fn regexp_literal_creation(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Lex all the tokens.
    let mut lexer = Lexer::new(REGEXP_LITERAL_CREATION);
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("RegExp Literal Creation (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static REGEXP_CREATION: &str = include_str!("bench_scripts/regexp_creation.js");

fn regexp_creation(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Lex all the tokens.
    let mut lexer = Lexer::new(REGEXP_CREATION);
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("RegExp (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static REGEXP_LITERAL: &str = include_str!("bench_scripts/regexp_literal.js");

fn regexp_literal(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Lex all the tokens.
    let mut lexer = Lexer::new(REGEXP_LITERAL);
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("RegExp Literal (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static REGEXP: &str = include_str!("bench_scripts/regexp.js");

fn regexp(c: &mut Criterion) {
    // Create new Realm and interpreter.
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    // Lex all the tokens.
    let mut lexer = Lexer::new(REGEXP);
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("RegExp (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static ARRAY_ACCESS: &str = include_str!("bench_scripts/array_access.js");

fn array_access(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(ARRAY_ACCESS);
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    c.bench_function("Array access (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static ARRAY_CREATE: &str = include_str!("bench_scripts/array_create.js");

fn array_creation(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(ARRAY_CREATE);
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    c.bench_function("Array creation (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static ARRAY_POP: &str = include_str!("bench_scripts/array_pop.js");

fn array_pop(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(ARRAY_POP);
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    c.bench_function("Array pop (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static STRING_CONCAT: &str = include_str!("bench_scripts/string_concat.js");

fn string_concat(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(STRING_CONCAT);
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    c.bench_function("String concatenation (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static STRING_COMPARE: &str = include_str!("bench_scripts/string_compare.js");

fn string_compare(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(STRING_COMPARE);
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    c.bench_function("String comparison (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static STRING_COPY: &str = include_str!("bench_scripts/string_copy.js");

fn string_copy(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(STRING_COPY);
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    c.bench_function("String copy (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static NUMBER_OBJECT_ACCESS: &str = include_str!("bench_scripts/number_object_access.js");

fn number_object_access(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(NUMBER_OBJECT_ACCESS);
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    c.bench_function("Number Object Access (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static BOOLEAN_OBJECT_ACCESS: &str = include_str!("bench_scripts/boolean_object_access.js");

fn boolean_object_access(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(BOOLEAN_OBJECT_ACCESS);
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    c.bench_function("Boolean Object Access (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static STRING_OBJECT_ACCESS: &str = include_str!("bench_scripts/string_object_access.js");

fn string_object_access(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(STRING_OBJECT_ACCESS);
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    c.bench_function("String Object Access (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static ARITHMETIC_OPERATIONS: &str = include_str!("bench_scripts/arithmetic_operations.js");

fn arithmetic_operations(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(ARITHMETIC_OPERATIONS);
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();

    c.bench_function("Arithmetic operations (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static CLEAN_JS: &str = include_str!("bench_scripts/clean_js.js");

fn clean_js(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let mut lexer = Lexer::new(CLEAN_JS);
    lexer.lex().expect("failed to lex");
    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();
    c.bench_function("Clean js (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static MINI_JS: &str = include_str!("bench_scripts/mini_js.js");

fn mini_js(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let mut lexer = Lexer::new(MINI_JS);
    lexer.lex().expect("failed to lex");
    let nodes = Parser::new(&lexer.tokens).parse_all().unwrap();
    c.bench_function("Mini js (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

criterion_group!(
    execution,
    create_realm,
    symbol_creation,
    for_loop_execution,
    fibonacci,
    array_access,
    array_creation,
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
