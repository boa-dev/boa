// Factorial function with recursion
//
// To test:
// 1. Set a breakpoint on line 8 (inside the function)
// 2. Press F5 to start debugging
// 3. Use Step In (F11) to follow the recursion
// 4. Observe the call stack growing with each recursive call
// 5. Inspect the 'n' parameter in each frame

function factorial(n) {
  if (n <= 1) {
    return 1; // Base case - set breakpoint here to see return
  }
  return n * factorial(n - 1); // Recursive case
}

console.log("Computing factorial(5)...");
const result = factorial(5);
console.log("factorial(5) =", result); // Should be 120

// Test with different values
console.log("factorial(3) =", factorial(3)); // Should be 6
console.log("factorial(7) =", factorial(7)); // Should be 5040

debugger; // Pause to inspect results

console.log("Done!");
