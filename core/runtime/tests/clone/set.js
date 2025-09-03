const set1 = new Set();

const v1 = new Uint8Array([1, 2]);

set1.add(v1);
set1.add('someValue');
set1.add(new Uint8Array([3, 4]));
set1.add(set1);  // Russell be damned!

const set2 = structuredClone(set1, {transfer: [v1.buffer]})
const [s2v1, v2, v3, v4] = [...set2.values()];

assert(s2v1 !== v1);
assertEq(v1.buffer.byteLength, 0);
assertArrayEqual(s2v1, [1, 2]);
assertEq(v2, 'someValue');
assertArrayEqual(v3, [3, 4]);
assertEq(set2, v4);
