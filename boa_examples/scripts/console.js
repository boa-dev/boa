console.log(1, "a", false, true, Promise.resolve(1), Symbol(1));

console.log(new Set([1, 2, 3, 4]));

console.log(
  new Map([
    [1, 2],
    ["a", "b"],
    [false, true],
  ]),
);

let a = [1];
a[1] = a;
console.log(a);

let b = { a: [1, 2, "a"] };
b["b"] = b;

console.log(b);

console.log(new Error("console error"));
