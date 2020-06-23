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

    // Lex all the tokens.
    let mut lexer = Lexer::new(black_box(OBJECT_CREATION));
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

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

    // Lex all the tokens.
    let mut lexer = Lexer::new(black_box(OBJECT_PROP_ACCESS_CONST));
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

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

    // Lex all the tokens.
    let mut lexer = Lexer::new(black_box(OBJECT_PROP_ACCESS_DYN));
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

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

    // Lex all the tokens.
    let mut lexer = Lexer::new(black_box(REGEXP_LITERAL_CREATION));
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

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

    // Lex all the tokens.
    let mut lexer = Lexer::new(black_box(REGEXP_CREATION));
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

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

    // Lex all the tokens.
    let mut lexer = Lexer::new(black_box(REGEXP_LITERAL));
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

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

    // Lex all the tokens.
    let mut lexer = Lexer::new(black_box(REGEXP));
    lexer.lex().expect("failed to lex");

    // Parse the AST nodes.
    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

    // Execute the parsed nodes, passing them through a black box, to avoid over-optimizing by the compiler
    c.bench_function("RegExp (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static ARRAY_ACCESS: &str = r#"
(function () {
    let testArr = [1,2,3,4,5];

    let res = testArr[2];

    return res;
})();
"#;

fn array_access(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(black_box(ARRAY_ACCESS));
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

    c.bench_function("Array access (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static ARRAY_CREATE: &str = r#"
(function(){
    let testArr = [];
    for (let a = 0; a <= 500; a++) {
        testArr[a] = ('p' + a);
    }

    return testArr;
})();
"#;

fn array_creation(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(black_box(ARRAY_CREATE));
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

    c.bench_function("Array creation (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static ARRAY_POP: &str = r#"
(function(){
    let testArray = [83, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32,
                     45, 93, 17, 28, 83, 62, 99, 36, 28, 93, 27, 29, 2828,
                     234, 23, 56, 32, 56, 67, 77, 32, 45, 93, 17, 28, 83, 62,
                     99, 36, 28, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67,
                     77, 32, 45, 93, 17, 28, 83, 62, 99, 36, 28, 93, 27, 29,
                     2828, 234, 23, 56, 32, 56, 67, 77, 32, 45, 93, 17, 28,
                     83, 62, 99, 36, 28, 93, 27, 29, 2828, 234, 23, 56, 32,
                     56, 67, 77, 32, 45, 93, 17, 28, 83, 62, 99, 36, 28, 93,
                     27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32, 45, 93,
                     17, 28, 83, 62, 99, 36, 28, 93, 27, 29, 2828, 234, 23,
                     56, 32, 56, 67, 77, 32, 45, 93, 17, 28, 83, 62, 99, 36,
                     28, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32,
                     45, 93, 17, 28, 83, 62, 99, 36, 28, 93, 27, 29, 2828, 234,
                     23, 56, 32, 56, 67, 77, 32, 45, 93, 17, 28, 83, 62, 99,
                     36, 28, 93, 27, 29, 2828, 234, 23, 56, 32, 56, 67, 77, 32,
                     45, 93, 17, 28, 83, 62, 99, 36, 28];

    while (testArray.length > 0) {
        testArray.pop();
    }

    return testArray;
})();
"#;

fn array_pop(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(black_box(ARRAY_POP));
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

    c.bench_function("Array pop (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static STRING_CONCAT: &str = r#"
(function(){
    var a = "hello";
    var b = "world";

    var c = a + b;
})();
"#;

fn string_concat(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(black_box(STRING_CONCAT));
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

    c.bench_function("String concatenation (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static STRING_COMPARE: &str = r#"
(function(){
    var a = "hello";
    var b = "world";

    var c = a == b;

    var d = b;
    var e = d == b;
})();
"#;

fn string_compare(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(black_box(STRING_COMPARE));
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

    c.bench_function("String comparison (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static STRING_COPY: &str = r#"
(function(){
    var a = "hello";
    var b = a;
})();
"#;

fn string_copy(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(black_box(STRING_COPY));
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

    c.bench_function("String copy (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static NUMBER_OBJECT_ACCESS: &str = r#"
new Number(
    new Number(
        new Number(
            new Number(100).valueOf() - 10.5
        ).valueOf() + 100
    ).valueOf() * 1.6
)
"#;

fn number_object_access(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(black_box(NUMBER_OBJECT_ACCESS));
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

    c.bench_function("Number Object Access (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static BOOLEAN_OBJECT_ACCESS: &str = r#"
new Boolean(
    !new Boolean(
        new Boolean(
            !(new Boolean(false).valueOf()) && (new Boolean(true).valueOf())
        ).valueOf()
    ).valueOf()
).valueOf()
"#;

fn boolean_object_access(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(black_box(BOOLEAN_OBJECT_ACCESS));
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

    c.bench_function("Boolean Object Access (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static STRING_OBJECT_ACCESS: &str = r#"
new String(
    new String(
        new String(
            new String('Hello').valueOf() + new String(", world").valueOf()
        ).valueOf() + '!'
    ).valueOf()
).valueOf()
"#;

fn string_object_access(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(black_box(STRING_OBJECT_ACCESS));
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

    c.bench_function("String Object Access (Execution)", move |b| {
        b.iter(|| black_box(&nodes).run(&mut engine).unwrap())
    });
}

static ARITHMETIC_OPERATIONS: &str = r#"
((2 + 2) ** 3 / 100 - 5 ** 3 * -1000) ** 2 + 100 - 8
"#;

fn arithmetic_operations(c: &mut Criterion) {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let mut lexer = Lexer::new(black_box(ARITHMETIC_OPERATIONS));
    lexer.lex().expect("failed to lex");

    let nodes = Parser::new(&black_box(lexer.tokens)).parse_all().unwrap();

    c.bench_function("Arithmetic operations (Execution)", move |b| {
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
);
criterion_main!(execution);
