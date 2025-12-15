// This script should take a few seconds to run.
const kIterationCount = 10_000;
const base = "abcdefghijklmnopqrstuvwxyz".repeat(1_000);

function main() {
  const k = base.split("a").length;
  console.log(k);
}
