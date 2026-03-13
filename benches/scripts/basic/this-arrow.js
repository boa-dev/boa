const obj = {
  value: 0,
  increment() {
    const add = () => {
      this.value += 1;
    };
    add();
  },
};
const start = Date.now();
for (let i = 0; i < 1000000; i++) {
  obj.increment();
}
