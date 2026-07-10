// Measures closure invocation overhead: closures are created once,
// then called repeatedly to isolate dispatch and variable lookup cost.
const kIterationCount = 100_000;

function makeCounter(start) {
  let count = start;
  return {
    increment: (n) => {
      count += n;
      return count;
    },
    decrement: (n) => {
      count -= n;
      return count;
    },
    value: () => count,
  };
}

const counterA = makeCounter(0);
const counterB = makeCounter(1000);
const counterC = makeCounter(-500);

function main() {
  let sum = 0;
  for (let i = 0; i < kIterationCount; i++) {
    sum += counterA.increment(1);
    sum += counterB.decrement(1);
    sum += counterC.value();
  }
  return sum;
}
