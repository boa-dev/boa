// Closures and scope test
//
// To test:
// 1. Set breakpoints on lines 11, 14, and 18
// 2. Press F5 to start debugging
// 3. Inspect variables to see closure capturing
// 4. Step through to see how closures maintain their environment

function createCounter(start) {
    let count = start;
    
    return function increment() {
        count++; // Set breakpoint here
        console.log("Count:", count);
        return count;
    };
}

const counter1 = createCounter(0);
const counter2 = createCounter(100);

counter1(); // Should print 1
counter1(); // Should print 2
counter2(); // Should print 101
counter1(); // Should print 3

debugger; // Inspect counter1 and counter2

console.log("Final call:");
console.log(counter2()); // Should print 102
