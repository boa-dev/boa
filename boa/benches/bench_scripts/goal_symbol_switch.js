function foo(regex, num) {}

let i = 0;
while (i < 1000000) {
  foo(/ab+c/, 5.0 / 5);
  i++;
}
