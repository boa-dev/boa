const kIterationCount = 1000;
const fmt = new Intl.DateTimeFormat("en-US", {
    dateStyle: "full",
    timeStyle: "short",
    timeZone: "UTC",
});

function main() {
  for (let i = 0; i < kIterationCount; i++) {
      fmt.resolvedOptions();
  }
}