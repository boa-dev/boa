// This script should take a few seconds to run.
const kIterationCount = 10_000;
const base = "abcdefghijklmnopqrstuvwxyz".repeat(10000000);

function main() {
  for (let i = 0; i < kIterationCount; i++) {
    base.slice(i * 100, i * 100 + 20000);
  }
}
