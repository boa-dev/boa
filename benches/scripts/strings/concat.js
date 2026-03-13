// Measures repeated string concatenation via the += operator,
// stressing string allocation and garbage collection throughput.
const kIterationCount = 10_000;
const fragments = [
  "hello",
  " ",
  "world",
  "! ",
  "foo",
  "bar",
  "baz",
  " ",
  "qux",
  "\n",
];

function main() {
  let result = "";
  for (let i = 0; i < kIterationCount; i++) {
    result += fragments[i % fragments.length];
  }
  return result.length;
}
