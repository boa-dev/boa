const add = (a, b) => a + b;
let sum = 0;
for (let i = 0; i < 1000000; i++) {
    sum = add(i, 1);
}