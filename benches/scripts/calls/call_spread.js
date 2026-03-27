function test(a, b, c) {
  return a + b + c;
}

const args = [1, 2, 3];

// Warmup
for (let i = 0; i < 1000; i++) {
  test(...args);
}

let sum = 0;
for (let i = 0; i < 10000000; i++) {
  sum += test(...args);
}

console.log("Spread Call Sum:", sum);
