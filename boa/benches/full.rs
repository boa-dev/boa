//! Benchmarks of whole program execution in Boa.

use boa::exec;
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

fn symbol_creation(c: &mut Criterion) {
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("Symbols (Full)", move |b| {
        b.iter(|| exec(black_box(SYMBOL_CREATION)))
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

fn for_loop(c: &mut Criterion) {
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("For loop (Full)", move |b| {
        b.iter(|| exec(black_box(FOR_LOOP)))
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
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("Fibonacci (Full)", move |b| {
        b.iter(|| exec(black_box(FIBONACCI)))
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
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("Object Creation (Full)", move |b| {
        b.iter(|| exec(black_box(OBJECT_CREATION)))
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
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("Static Object Property Access (Full)", move |b| {
        b.iter(|| exec(black_box(OBJECT_PROP_ACCESS_CONST)))
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
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("Dynamic Object Property Access (Full)", move |b| {
        b.iter(|| exec(black_box(OBJECT_PROP_ACCESS_DYN)))
    });
}

static REGEXP_LITERAL_CREATION: &str = r#"
(function () {
    let regExp = /hello/i;

    return regExp;
})();
"#;

fn regexp_literal_creation(c: &mut Criterion) {
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("RegExp Literal Creation (Full)", move |b| {
        b.iter(|| exec(black_box(REGEXP_LITERAL_CREATION)))
    });
}

static REGEXP_CREATION: &str = r#"
(function () {
    let regExp = new RegExp('hello', 'i');

    return regExp;
})();
"#;

fn regexp_creation(c: &mut Criterion) {
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("RegExp (Full)", move |b| {
        b.iter(|| exec(black_box(REGEXP_CREATION)))
    });
}

static REGEXP_LITERAL: &str = r#"
(function () {
    let regExp = /hello/i;

    return regExp.test("Hello World");
})();
"#;

fn regexp_literal(c: &mut Criterion) {
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("RegExp Literal (Full)", move |b| {
        b.iter(|| exec(black_box(REGEXP_LITERAL)))
    });
}

static REGEXP: &str = r#"
(function () {
    let regExp = new RegExp('hello', 'i');

    return regExp.test("Hello World");
})();
"#;

fn regexp(c: &mut Criterion) {
    // Execute the code by taking into account realm creation, lexing and parsing
    c.bench_function("RegExp (Full)", move |b| b.iter(|| exec(black_box(REGEXP))));
}

static ARRAY_ACCESS: &str = r#"
(function () {
    let testArr = [1,2,3,4,5];

    let res = testArr[2];

    return res;
})();
"#;

fn array_access(c: &mut Criterion) {
    c.bench_function("Array access (Full)", move |b| {
        b.iter(|| exec(black_box(ARRAY_ACCESS)))
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
    c.bench_function("Array creation (Full)", move |b| {
        b.iter(|| exec(black_box(ARRAY_CREATE)))
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
    c.bench_function("Array pop (Full)", move |b| {
        b.iter(|| exec(black_box(ARRAY_POP)))
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
    c.bench_function("String concatenation (Full)", move |b| {
        b.iter(|| exec(black_box(STRING_CONCAT)))
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
    c.bench_function("String comparison (Full)", move |b| {
        b.iter(|| exec(black_box(STRING_COMPARE)))
    });
}

static STRING_COPY: &str = r#"
(function(){
    var a = "hello";
    var b = a;
})();
"#;

fn string_copy(c: &mut Criterion) {
    c.bench_function("String copy (Full)", move |b| {
        b.iter(|| exec(black_box(STRING_COPY)))
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
    c.bench_function("Number Object Access (Full)", move |b| {
        b.iter(|| exec(black_box(NUMBER_OBJECT_ACCESS)))
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
    c.bench_function("Boolean Object Access (Full)", move |b| {
        b.iter(|| exec(black_box(BOOLEAN_OBJECT_ACCESS)))
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
    c.bench_function("String Object Access (Full)", move |b| {
        b.iter(|| exec(black_box(STRING_OBJECT_ACCESS)))
    });
}

static ARITHMETIC_OPERATIONS: &str = r#"
((2 + 2) ** 3 / 100 - 5 ** 3 * -1000) ** 2 + 100 - 8
"#;

fn arithmetic_operations(c: &mut Criterion) {
    c.bench_function("Arithmetic operations (Full)", move |b| {
        b.iter(|| exec(black_box(ARITHMETIC_OPERATIONS)))
    });
}

static CLEAN_JS: &str = r#"
!function () {
	var M = new Array();
	for (i = 0; i < 100; i++) {
		M.push(Math.floor(Math.random() * 100));
	}
	var test = [];
	for (i = 0; i < 100; i++) {
		if (M[i] > 50) {
			test.push(M[i]);
		}
	}
	test.forEach(elem => {
        0
    });
}();
"#;

fn clean_js(c: &mut Criterion) {
    c.bench_function("Clean js (Full)", move |b| {
        b.iter(|| exec(black_box(CLEAN_JS)))
    });
}

static MINI_JS: &str = r#"
!function(){var r=new Array();for(i=0;i<100;i++)r.push(Math.floor(100*Math.random()));var a=[];for(i=0;i<100;i++)r[i]>50&&a.push(r[i]);a.forEach(i=>{0})}();
"#;

fn mini_js(c: &mut Criterion) {
    c.bench_function("Mini js (Full)", move |b| {
        b.iter(|| exec(black_box(MINI_JS)))
    });
}

criterion_group!(
    full,
    symbol_creation,
    for_loop,
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
criterion_main!(full);
