const kIterationCount = 1000;
const pr = new Intl.PluralRules("en");

function main() {
  for (let i = 0; i < kIterationCount; i++) {
    pr.select(i);
  }
}
