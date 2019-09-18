let num = 12;

function fib(n) {
  if (n <= 1) return 1;
  return fib(n - 1) + fib(n - 2);
}

let res = fib(num);

res;

// (2 - 1 = 1) + (2 - 2 = rt 1)
