// Async/Promise test (if Boa supports it)
//
// To test:
// 1. Set breakpoints on lines 8, 13, and 18
// 2. Press F5 to start debugging
// 3. Step through to see promise execution
// Note: This may not work if Boa's async support is limited

function delay(ms) {
  return new Promise((resolve) => {
    // In a full implementation, this would actually delay
    console.log("Promise created");
    resolve();
  });
}

async function asyncTest() {
  console.log("Start");

  debugger; // Pause before await

  await delay(100);

  console.log("After delay");
  return "Done!";
}

asyncTest().then((result) => {
  console.log("Result:", result);
});

console.log("Main thread continues");
