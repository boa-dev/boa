// Verify this is hoisted outside the loop.
for (let i = 0; i < 100; i++) {}

const n = 100;
for (let i = 0; i < n; i++) {}

// This should also be hoisted since it's const.
const z = 100;

function bar() {
  for (let i = 0; i < z; i++) {}
}

bar();

// This should NOT be hoisted since it's a mutable binding.
let x = 100;

function foo() {
  for (let i = 0; i < x; i++) {}
}

foo();
