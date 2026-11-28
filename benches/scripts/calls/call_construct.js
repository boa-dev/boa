// Repeated construction isolates the constructor call path.
const iterationCount = 10_000;

function Empty() {}

function main() {
  let instance;
  for (let i = 0; i < iterationCount; i++) {
    instance = new Empty();
  }
  return instance;
}
