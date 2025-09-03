const m1 = new Map();

const k1 = new Uint8Array([1, 2]);
const k2 = 'someKey';
const k3 = 5;

m1.set(k1, 'hello');
m1.set(k2, 'world');
m1.set(k3, k1);

const m2 = structuredClone(m1, {transfer: [k1.buffer]})
console.log([...m2.entries()]);
const [m2k1, m2k2, m2k3] = [...m2.keys()];

assert(k1 !== m2k1);
console.log(m2k1);
assertArrayEqual(m2k1, [1, 2]);
assert(m2k1 === m2[5]);
