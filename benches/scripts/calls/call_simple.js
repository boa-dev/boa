function test() { return 1; }
// Warmup
for (let i = 0; i < 1000; i++) { test(); }
let sum = 0;
for (let i = 0; i < 10000000; i++) {
    sum += test();
}
console.log(sum);
