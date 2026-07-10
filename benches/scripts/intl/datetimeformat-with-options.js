const kIterationCount = 100;

function main() {
  for (let i = 0; i < kIterationCount; i++) {
    new Intl.DateTimeFormat("en", {
      year: "numeric",
      month: "long",
      day: "numeric",
    });
    new Intl.DateTimeFormat("en", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  }
}
