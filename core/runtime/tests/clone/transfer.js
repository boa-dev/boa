// https://developer.mozilla.org/en-US/docs/Web/API/Window/structuredClone#transferring_an_object

// Create an ArrayBuffer with a size in bytes
const buffer = new ArrayBuffer(16);

const object1 = {
  buffer,
};

// Clone the object containing the buffer, and transfer it
const object2 = structuredClone(object1, { transfer: [buffer] });

// Create an array from the cloned buffer
const int32View2 = new Int32Array(object2.buffer);
int32View2[0] = 42;
assertEq(int32View2[0], 42);

// Creating an array from the original buffer throws a TypeError
assertThrows(() => {
  const int32View1 = new Int32Array(object1.buffer);
});
