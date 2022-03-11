//access custom global variable
console.log("Custom global: " + customstring);

//call a custom global function with arguments
console.log("Custom function: " + rusty_hello("Boa! Boa!"));

//access a custom global object and call a member function of that object
let a = 5;
let b = 5;
let result = rusty_obj.add(a, b);
console.log("Custom object: Result from rusty_obj.add() : " + result);
