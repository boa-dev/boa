// Exception handling test
//
// To test:
// 1. Enable "Pause on Exceptions" in the debug toolbar
// 2. Press F5 to start debugging
// 3. Execution should pause when the exception is thrown
// 4. Inspect the call stack at the exception point

function divide(a, b) {
  if (b === 0) {
    throw new Error("Division by zero!");
  }
  return a / b;
}

function calculate() {
  console.log("10 / 2 =", divide(10, 2));
  console.log("20 / 4 =", divide(20, 4));

  debugger; // Pause before the error

  console.log("10 / 0 =", divide(10, 0)); // This will throw
}

try {
  calculate();
} catch (error) {
  console.log("Caught error:", error.message);
}

console.log("Program continues after exception");
