// ===== BASIC TEST =====
function basicTest() {
    console.log(arguments);
}
basicTest('first', 'second');
basicTest('first', 'second', 42, null, true);


// ===== OBJECTS & ARRAYS =====
function objectTest() {
    const obj = {
        name: "Sushi",
        age: 21,
        nested: {
            arr: [1, 2, { deep: "value" }]
        }
    };

    const sparseArray = [];
    sparseArray[2] = "third";

    console.log(arguments);
}
objectTest({ a: 1 }, [1, 2, 3], [], { x: { y: { z: 1 } } });


// ===== CIRCULAR REFERENCES =====
function circularTest() {
    const a = {};
    const b = { a };
    a.b = b; // circular

    console.log(arguments);
}
circularTest({ foo: "bar" }, (function () {
    const obj = {};
    obj.self = obj;
    return obj;
})());


// ===== FUNCTIONS & CLASSES =====
function functionTest() {
    function inner() { }
    const arrow = () => { };
    class MyClass { }

    console.log(arguments);
}
functionTest(function named() { }, () => 123, class TestClass { });
functionTest(class TestClass2 { }, function named2() { }, () => 456);


// ===== SPECIAL TYPES =====
function specialTypesTest() {
    console.log(arguments);
}
specialTypesTest(
    new Date(),
    /regex-test/gi,
    BigInt(9007199254740991),
    Symbol("mysymbol")
);


// ===== MAP, SET =====
function collectionTest() {
    const map = new Map();
    map.set("key1", "value1");
    map.set("key2", { nested: true });

    const set = new Set([1, 2, 3]);

    console.log(arguments);
}
collectionTest(
    new Map([["a", 1], ["b", 2]]),
    new Set([1, 2, 3])
);


// ===== TYPED ARRAYS & BUFFERS =====
function typedArrayTest() {
    console.log(arguments);
}
typedArrayTest(
    new Uint8Array([1, 2, 3]),
    new Int32Array([1000, 2000])
);


// ===== ERRORS =====
function errorTest() {
    console.log(arguments);
}
errorTest(new Error("Something went wrong"));


// ===== VERY LONG STRING =====
function longStringTest() {
    const longString = "x".repeat(200);
    console.log(arguments);
}
longStringTest("short", "x".repeat(1000));


const deep = {};
// ===== DEEP NESTING =====
function deepNestTest() {
    const deep = {};
    let current = deep;
    for (let i = 0; i < 20; i++) {
        current.next = {};
        current = current.next;
    }

    console.log(arguments);
}
deepNestTest(deep);


const circular = {};
// ===== MIXED EVERYTHING =====
function everythingTest() {
    const circular = {};
    circular.self = circular;

    console.log(arguments);
}
everythingTest(
    1,
    "string",
    true,
    null,
    undefined,
    Symbol("sym"),
    BigInt(123),
    [1, { a: 2 }, [3]],
    { nested: { deeper: { value: 10 } } },
    function foo() { },
    () => { },
    new Date(),
    /abc/,
    new Map([["k", "v"]]),
    new Set([1, 2]),
    circular
);