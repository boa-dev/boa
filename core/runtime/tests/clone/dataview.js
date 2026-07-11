// https://developer.mozilla.org/en-US/docs/Web/API/Window/structuredClone

{
  // Test basic DataView cloning
  const buffer = new ArrayBuffer(16);
  const dataview = new DataView(buffer);
  dataview.setInt8(0, 42);

  const clone = structuredClone(dataview);

  assertNEq(clone, dataview);
  assertNEq(clone.buffer, dataview.buffer);
  assertEq(clone.byteLength, 16);
  assertEq(clone.byteOffset, 0);
  assertEq(clone.getInt8(0), 42);

  // modifying clone doesn't affect original
  clone.setInt8(0, 10);
  assertEq(dataview.getInt8(0), 42);
}

{
  // Test DataView with byteOffset and byteLength
  const buffer = new ArrayBuffer(32);
  const dataview = new DataView(buffer, 8, 16);
  dataview.setInt16(0, 1234);

  const clone = structuredClone(dataview);

  assertNEq(clone, dataview);
  assertNEq(clone.buffer, dataview.buffer);
  assertEq(clone.byteLength, 16);
  assertEq(clone.byteOffset, 8);
  assertEq(clone.getInt16(0), 1234);
}

{
  // Test transferring the underlying buffer of a DataView
  const buffer = new ArrayBuffer(16);
  const dataview = new DataView(buffer, 4, 8);
  dataview.setInt32(0, 98765);

  const object1 = {
    dataview,
  };

  const object2 = structuredClone(object1, { transfer: [buffer] });

  assert(object2.dataview !== dataview);
  assertEq(object1.dataview.byteLength, 0); // Original dataview is detached
  assertEq(object2.dataview.byteLength, 8);
  assertEq(object2.dataview.byteOffset, 4);
  assertEq(object2.dataview.getInt32(0), 98765);
}
