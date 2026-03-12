function outer() {
  let x = 1;
  function middle() {
    let y = 2;
    function inner() {
      return x + y;
    }
    return inner;
  }
  return middle;
}
let f = outer()();

function main() {
  for (let n = 0; n < 10_000_000; n++) {
    f();
  }
}
