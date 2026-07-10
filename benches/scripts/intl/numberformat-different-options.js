const kIterationCount = 100;

function main() {
  for (let i = 0; i < kIterationCount; i++) {
    new Intl.NumberFormat("en", { style: "currency", currency: "USD" });
    new Intl.NumberFormat("en", { style: "percent" });
    new Intl.NumberFormat("en", { notation: "scientific" });
  }
}
