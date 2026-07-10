// Measures closure creation overhead: allocation of function objects
// and captured variable environments inside a hot loop.
const kIterationCount = 100_000;

function main() {
  let sum = 0;
  for (let i = 0; i < kIterationCount; i++) {
    const x = i;
    const y = i * 2;
    const closure = (z) => x + y + z;
    sum += closure(i + 1);
  }
  return sum;
}
