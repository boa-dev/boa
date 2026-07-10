const kIterationCount = 1000;
const collator = new Intl.Collator("en");

function main() {
  for (let i = 0; i < kIterationCount; i++) {
    collator.compare("apple", "banana");
  }
}
