// This script should take a few seconds to run.
const kIterationCount = 5_000_000;
const base = "abcdefghijklmnopqrstuvwxyz".repeat(10000000);

const start = Date.now();
for (let i = 0; i < kIterationCount; i++) {
  base.slice(i * 100, i * 100 + 20000);
}
const end = Date.now();

console.log(end - start);
