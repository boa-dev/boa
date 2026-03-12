const kIterationCount = 1000;
const lf = new Intl.ListFormat("en");
const list = ["apple", "banana", "cherry"];

function main() {
  for (let i = 0; i < kIterationCount; i++) {
    lf.format(list);
  }
}
