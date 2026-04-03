async function test() {
  return 1;
}

(async function () {
  // Warmup
  for (let i = 0; i < 1000; i++) {
    await test();
  }

  let sum = 0;
  for (let i = 0; i < 1000000; i++) { // Using 1M for async due to Promise overhead
    sum += await test();
  }

  console.log("Async Call Sum:", sum);
})();
