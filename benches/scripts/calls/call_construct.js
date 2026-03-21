function Test() {
  this.val = 1;
}
// Warmup
for (let i = 0; i < 1000; i++) {
  new Test();
}
let sum = 0;
for (let i = 0; i < 1000000; i++) {
  let obj = new Test();
  sum += obj.val;
}
console.log("Construct Call Sum:", sum);
