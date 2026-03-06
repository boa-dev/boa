const kIterationCount = 1000;
const dtf = new Intl.DateTimeFormat("en");
const date = new Date(2024, 0, 1);

function main() {
  for (let i = 0; i < kIterationCount; i++) {
    dtf.format(date);
  }
}
