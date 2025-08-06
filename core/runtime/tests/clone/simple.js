// From https://developer.mozilla.org/en-US/docs/Web/API/Window/structuredClone#description

// Create an object with a value and a circular reference to itself.
const original = { name: "MDN" };
original.itself = original;

// Clone it
const clone = structuredClone(original);

assertNEq(clone, original); // the objects are not the same (not same identity)
assertEq(clone.name, "MDN"); // they do have the same values
assertEq(clone.itself, clone); // and the circular reference is preserved
