function f() {}

function main() {
  for (let i = 0; i < 100_000; i++) {
    f();
  }
}
