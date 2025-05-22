console.log("Hello world");
console.table(JSON.stringify({ a: 1, b: 2 }));
console.warn("This is a warning");
console.error("This is an error");
console.info("This is an info message");
console.assert(1 === 2, "This is an assertion message");


function foo() {
    function bar() {
        console.trace();
    }
    bar();
}

foo();
