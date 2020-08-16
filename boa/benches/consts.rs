pub static EXPRESSION: &str = r#"
1 + 1 + 1 + 1 + 1 + 1 / 1 + 1 + 1 * 1 + 1 + 1 + 1;
"#;

pub static HELLO_WORLD: &str = "let foo = 'hello world!'; foo;";

pub static LONG_REPETITION: &str = r#"
for (let a = 10; a < 100; a++) {
    if (a < 10) {
        console.log("impossible D:");
    } else if (a < 50) {
        console.log("starting");
    } else {
        console.log("finishing");
    }
}
"#;

pub static GOAL_SYMBOL_SWITCH: &str = r#"
function foo(regex, num) {}

let i = 0;
while (i < 1000000) {
    foo(/ab+c/, 5.0/5);
    i++;
}
"#;

pub static SYMBOL_CREATION: &str = r#"
(function () {
    return Symbol();
})();
"#;

pub static FOR_LOOP: &str = r#"
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

pub static FIBONACCI: &str = r#"
(function () {
    let num = 12;

    function fib(n) {
        if (n <= 1) return 1;
        return fib(n - 1) + fib(n - 2);
    }

    return fib(num);
})();
"#;

pub static OBJECT_CREATION: &str = r#"
(function () {
    let test = {
        my_prop: "hello",
        another: 65,
    };

    return test;
})();
"#;

pub static OBJECT_PROP_ACCESS_CONST: &str = r#"
(function () {
    let test = {
        my_prop: "hello",
        another: 65,
    };

    return test.my_prop;
})();
"#;

pub static OBJECT_PROP_ACCESS_DYN: &str = r#"
(function () {
    let test = {
        my_prop: "hello",
        another: 65,
    };

    return test["my" + "_prop"];
})();
"#;

pub static REGEXP_LITERAL_CREATION: &str = r#"
(function () {
    let regExp = /hello/i;

    return regExp;
})();
"#;

pub static REGEXP_CREATION: &str = r#"
(function () {
    let regExp = new RegExp('hello', 'i');

    return regExp;
})();
"#;

pub static REGEXP_LITERAL: &str = r#"
(function () {
    let regExp = /hello/i;

    return regExp.test("Hello World");
})();
"#;

pub static REGEXP: &str = r#"
(function () {
    let regExp = new RegExp('hello', 'i');

    return regExp.test("Hello World");
})();
"#;

pub static ARRAY_ACCESS: &str = r#"
(function () {
    let testArr = [1,2,3,4,5];

    let res = testArr[2];

    return res;
})();
"#;

pub static ARRAY_CREATE: &str = r#"
(function(){
    let testArr = [];
    for (let a = 0; a <= 500; a++) {
        testArr[a] = ('p' + a);
    }

    return testArr;
})();
"#;

pub static ARRAY_POP: &str = r#"
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

pub static STRING_CONCAT: &str = r#"
(function(){
    var a = "hello";
    var b = "world";

    var c = a + b;
})();
"#;

pub static STRING_COMPARE: &str = r#"
(function(){
    var a = "hello";
    var b = "world";

    var c = a == b;

    var d = b;
    var e = d == b;
})();
"#;

pub static STRING_COPY: &str = r#"
(function(){
    var a = "hello";
    var b = a;
})();
"#;

pub static NUMBER_OBJECT_ACCESS: &str = r#"
new Number(
    new Number(
        new Number(
            new Number(100).valueOf() - 10.5
        ).valueOf() + 100
    ).valueOf() * 1.6
)
"#;

pub static BOOLEAN_OBJECT_ACCESS: &str = r#"
new Boolean(
    !new Boolean(
        new Boolean(
            !(new Boolean(false).valueOf()) && (new Boolean(true).valueOf())
        ).valueOf()
    ).valueOf()
).valueOf()
"#;

pub static STRING_OBJECT_ACCESS: &str = r#"
new String(
    new String(
        new String(
            new String('Hello').valueOf() + new String(", world").valueOf()
        ).valueOf() + '!'
    ).valueOf()
).valueOf()
"#;

pub static ARITHMETIC_OPERATIONS: &str = r#"
((2 + 2) ** 3 / 100 - 5 ** 3 * -1000) ** 2 + 100 - 8
"#;
