// Measures property access performance across varying object shapes:
// monomorphic (single shape), polymorphic (few shapes), and
// megamorphic (many shapes) lookup patterns.
const kIterationCount = 50_000;

// Monomorphic: always the same object shape.
function monoAccess(obj) {
  return obj.x + obj.y + obj.z;
}

// Polymorphic: receives objects with a few different shapes.
function polyAccess(obj) {
  return obj.value + obj.id;
}

// Test data: objects with distinct shapes.
const mono = { x: 1, y: 2, z: 3 };
const polyA = { value: 10, id: 1, a: "extra" };
const polyB = { id: 2, b: true, value: 20 };
const polyC = { c: null, d: 4, value: 30, id: 3 };
const polyD = { value: 40, id: 4, e: [1, 2] };

// Megamorphic: 20 objects, each with a unique shape.
const megaObjects = [];
for (let i = 0; i < 20; i++) {
  const obj = { value: i * 100, id: i };
  for (let j = 0; j < i; j++) {
    obj["prop" + j] = j;
  }
  megaObjects.push(obj);
}

function main() {
  let sum = 0;

  // Monomorphic access (single shape, hot path).
  for (let i = 0; i < kIterationCount; i++) {
    sum += monoAccess(mono);
  }

  // Polymorphic access (4 shapes rotating).
  const polyObjs = [polyA, polyB, polyC, polyD];
  for (let i = 0; i < kIterationCount; i++) {
    sum += polyAccess(polyObjs[i % 4]);
  }

  // Megamorphic access (20 unique shapes).
  for (let i = 0; i < kIterationCount; i++) {
    sum += polyAccess(megaObjects[i % 20]);
  }

  return sum;
}
