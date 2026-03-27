class Test {
  constructor(a, b) {
    this.a = a;
    this.b = b;
  }
}

// Warmup
for (let i = 0; i < 1000; i++) {
  new Test(i, i + 1);
}

const start = Date.now();
for (let i = 0; i < 1000000; i++) {
  new Test(i, i + 1);
}
const end = Date.now();

console.log("Time taken for 1M instantiations: " + (end - start) + "ms");
