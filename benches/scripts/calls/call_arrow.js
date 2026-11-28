// Keep the function body minimal so repeated calls dominate the benchmark.
const iterationCount = 10_000;
const identity = (value) => value;

function main() {
  let result = 0;
  for (let i = 0; i < iterationCount; i++) {
    result += identity(i);
  }
  return result;
}
