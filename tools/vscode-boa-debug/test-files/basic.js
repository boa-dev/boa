// Basic debugging test
//
// To test:
// 1. Set a breakpoint on line 7 (the console.log line)
// 2. Press F5 to start debugging
// 3. Execution should pause at the breakpoint
// 4. Inspect variables in the Variables panel
// 5. Use Step Over (F10) to continue

function greet(name) {
  const message = "Hello, " + name + "!";
  console.log(message);
  return message;
}

const result = greet("World");
console.log("Result:", result);

// Add a debugger statement
debugger; // Execution should pause here

console.log("Program finished");
