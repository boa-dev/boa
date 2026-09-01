var obj = {
  [Symbol.asyncIterator]() {
    return {
      next() {
        throw reason;
      },
    };
  },
};

async function* gen() {
  yield* obj;
}
